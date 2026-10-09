use crate::canvas::Canvas;
use crate::color::{hsv, Rgb};
use crate::scene::{Key, Scene};

const INSIDE: Rgb = Rgb(30, 60, 150);

pub struct Mandelbrot {
    w: usize,
    h: usize,
    cx: f64,
    cy: f64,
    scale: f64,
    julia: bool,
    angle: f64,
    max_iter: u32,
    pix: Vec<Option<Rgb>>,
    dirty: bool,
}

fn escape(mut zr: f64, mut zi: f64, cr: f64, ci: f64, max: u32) -> u32 {
    let mut n = 0;
    while n < max {
        let (r2, i2) = (zr * zr, zi * zi);
        if r2 + i2 > 4.0 {
            return n;
        }
        zi = 2.0 * zr * zi + ci;
        zr = r2 - i2 + cr;
        n += 1;
    }
    max
}

fn in_main_bulbs(x: f64, y: f64) -> bool {
    let q = (x - 0.25) * (x - 0.25) + y * y;
    q * (q + (x - 0.25)) <= 0.25 * y * y || (x + 1.0) * (x + 1.0) + y * y <= 0.0625
}

impl Mandelbrot {
    pub fn new() -> Self {
        let mut m = Mandelbrot {
            w: 0,
            h: 0,
            cx: -0.5,
            cy: 0.0,
            scale: 1.25,
            julia: false,
            angle: 0.6,
            max_iter: 60,
            pix: Vec::new(),
            dirty: true,
        };
        m.default_view();
        m
    }

    fn default_view(&mut self) {
        if self.julia {
            self.cx = 0.0;
            self.cy = 0.0;
            self.scale = 1.5;
        } else {
            self.cx = -0.5;
            self.cy = 0.0;
            self.scale = 1.25;
        }
        self.dirty = true;
    }

    fn julia_c(&self) -> (f64, f64) {
        (0.7885 * self.angle.cos(), 0.7885 * self.angle.sin())
    }

    fn iterations_for_zoom(&self) -> u32 {
        let zoom_levels = (1.25 / self.scale).log2().max(0.0);
        let cap = if self.julia { 120.0 } else { 600.0 };
        (60.0 + 30.0 * zoom_levels).min(cap) as u32
    }

    fn compute(&mut self) {
        let (w, h) = (self.w, self.h);
        if w == 0 || h == 0 {
            return;
        }
        self.max_iter = self.iterations_for_zoom();
        let max = self.max_iter;
        let aspect = w as f64 / h as f64;
        let (jr, ji) = self.julia_c();
        self.pix.clear();
        self.pix.reserve(w * h);
        for py in 0..h {
            let y0 = self.cy + (py as f64 / h as f64 - 0.5) * 2.0 * self.scale;
            for px in 0..w {
                let x0 = self.cx + (px as f64 / w as f64 - 0.5) * 2.0 * self.scale * aspect;
                let n = if self.julia {
                    escape(x0, y0, jr, ji, max)
                } else if in_main_bulbs(x0, y0) {
                    max
                } else {
                    escape(0.0, 0.0, x0, y0, max)
                };
                let color = if n >= max {
                    Some(INSIDE)
                } else if n % 2 == 0 {
                    Some(hsv(n as f64 * 9.0, 0.8, 1.0))
                } else {
                    None
                };
                self.pix.push(color);
            }
        }
        self.dirty = false;
    }
}

impl Scene for Mandelbrot {
    fn name(&self) -> &'static str {
        "mandelbrot"
    }

    fn status(&self) -> String {
        let mode = if self.julia { "Julia" } else { "Mandelbrot" };
        format!(
            "{} | center ({:.6}, {:.6}) zoom {:.1e} iter {} | arrows: pan  +/-: zoom  j: julia/mandelbrot",
            mode,
            self.cx,
            self.cy,
            1.25 / self.scale,
            self.max_iter
        )
    }

    fn resize(&mut self, px_w: usize, px_h: usize) {
        self.w = px_w;
        self.h = px_h;
        self.dirty = true;
    }

    fn reset(&mut self) {
        self.default_view();
    }

    fn update(&mut self, dt: f64) {
        if self.julia {
            self.angle += dt * 0.3;
            self.dirty = true;
        }
        if self.dirty {
            self.compute();
        }
    }

    fn draw(&self, canvas: &mut Canvas) {
        if self.pix.len() != self.w * self.h {
            return;
        }
        for y in 0..self.h {
            for x in 0..self.w {
                if let Some(c) = self.pix[y * self.w + x] {
                    canvas.set(x as i32, y as i32, c);
                }
            }
        }
    }

    fn key(&mut self, key: Key) {
        let step = self.scale * 0.15;
        match key {
            Key::Left => self.cx -= step,
            Key::Right => self.cx += step,
            Key::Up => self.cy -= step,
            Key::Down => self.cy += step,
            Key::Char('+') | Key::Char('=') => self.scale = (self.scale * 0.8).max(1e-13),
            Key::Char('-') | Key::Char('_') => self.scale = (self.scale / 0.8).min(4.0),
            Key::Char('j') => {
                self.julia = !self.julia;
                self.default_view();
            }
            _ => return,
        }
        self.dirty = true;
    }
}
