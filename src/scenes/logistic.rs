use crate::canvas::Canvas;
use crate::color::{hsv, Rgb, WHITE};
use crate::hdr::{lin, Hdr};
use crate::scene::{Key, Scene};

const X0: f64 = 0.4;
const WARM: usize = 500;
const SAMPLES: usize = 500;
const R_LOW: f64 = 1.0;
const R_HIGH: f64 = 4.0;

pub struct Logistic {
    w: usize,
    h: usize,
    r_min: f64,
    r_max: f64,
    cursor: f64,
    hdr: Hdr,
    dirty: bool,
}

fn lyapunov(r: f64) -> f64 {
    let mut x = X0;
    for _ in 0..500 {
        x = r * x * (1.0 - x);
    }
    let n = 4000;
    let mut total = 0.0;
    for _ in 0..n {
        x = r * x * (1.0 - x);
        total += (r * (1.0 - 2.0 * x)).abs().max(1e-12).ln();
    }
    total / n as f64
}

impl Logistic {
    pub fn new() -> Self {
        Logistic {
            w: 0,
            h: 0,
            r_min: 2.5,
            r_max: 4.0,
            cursor: 3.7,
            hdr: Hdr::new(),
            dirty: true,
        }
    }

    fn column_of(&self, r: f64) -> i32 {
        let t = (r - self.r_min) / (self.r_max - self.r_min);
        (t * (self.w.max(2) - 1) as f64).round() as i32
    }

    fn y_of(&self, x: f64) -> usize {
        (((1.0 - x) * (self.h - 1) as f64).round().max(0.0) as usize).min(self.h - 1)
    }

    fn compute(&mut self) {
        self.hdr.clear();
        if self.w < 2 || self.h < 2 {
            return;
        }
        for px in 0..self.w {
            let t = px as f64 / (self.w - 1) as f64;
            let r = self.r_min + (self.r_max - self.r_min) * t;
            let mut x = X0;
            for _ in 0..WARM {
                x = r * x * (1.0 - x);
            }
            let color = lin(hsv(200.0 + 150.0 * t, 0.75, 1.0), 0.05);
            for _ in 0..SAMPLES {
                x = r * x * (1.0 - x);
                let y = self.y_of(x);
                self.hdr.add_px(px, y, color);
            }
        }
        self.dirty = false;
    }
}

impl Scene for Logistic {
    fn name(&self) -> &'static str {
        "logistic"
    }

    fn status(&self) -> String {
        let l = lyapunov(self.cursor);
        format!(
            "Logistic map | r = {:.5}  lyapunov {:+.3} ({}) | left/right: move cursor  +/-: zoom",
            self.cursor,
            l,
            if l > 0.0 { "chaos" } else { "order" }
        )
    }

    fn resize(&mut self, px_w: usize, px_h: usize) {
        self.w = px_w;
        self.h = px_h;
        self.hdr.resize(px_w, px_h);
        self.dirty = true;
    }

    fn reset(&mut self) {
        self.r_min = 2.5;
        self.r_max = 4.0;
        self.cursor = 3.7;
        self.dirty = true;
    }

    fn update(&mut self, _dt: f64) {
        if self.dirty {
            self.compute();
        }
    }

    fn draw(&self, canvas: &mut Canvas) {
        if self.w < 2 || self.h < 2 {
            return;
        }
        self.hdr.blit(canvas, 3.0, 0.5);
        let cx = self.column_of(self.cursor);
        let guide = Rgb(110, 110, 110);
        for y in (0..self.h as i32).step_by(3) {
            canvas.set(cx, y, guide);
        }
        let mut x = X0;
        for _ in 0..WARM {
            x = self.cursor * x * (1.0 - x);
        }
        for _ in 0..SAMPLES {
            x = self.cursor * x * (1.0 - x);
            canvas.set(cx, self.y_of(x) as i32, WHITE);
        }
    }

    fn key(&mut self, key: Key) {
        let step = (self.r_max - self.r_min) / self.w.max(1) as f64 * 2.0;
        match key {
            Key::Left => self.cursor = (self.cursor - step).max(self.r_min),
            Key::Right => self.cursor = (self.cursor + step).min(self.r_max),
            Key::Char('+') | Key::Char('=') => {
                let c = self.cursor;
                self.r_min = c - (c - self.r_min) * 0.6;
                self.r_max = c + (self.r_max - c) * 0.6;
                self.dirty = true;
            }
            Key::Char('-') | Key::Char('_') => {
                let c = self.cursor;
                self.r_min = (c - (c - self.r_min) / 0.6).max(R_LOW);
                self.r_max = (c + (self.r_max - c) / 0.6).min(R_HIGH);
                self.dirty = true;
            }
            _ => {}
        }
    }
}
