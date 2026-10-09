use crate::canvas::Canvas;
use crate::color::hsv;
use crate::hdr::{lin, Hdr};
use crate::rng::{seed_from_time, Rng};
use crate::scene::{Key, Scene};

const GLIDER: [(usize, usize); 5] = [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];

pub struct Life {
    w: usize,
    h: usize,
    cells: Vec<u8>,
    next: Vec<u8>,
    hdr: Hdr,
    acc: f64,
    rate: f64,
    generation: u64,
    pop: usize,
    stale: u32,
    rng: Rng,
}

fn alive(c: &[u8], w: usize, x: usize, y: usize) -> u8 {
    (c[y * w + x] > 0) as u8
}

impl Life {
    pub fn new() -> Self {
        Life {
            w: 0,
            h: 0,
            cells: Vec::new(),
            next: Vec::new(),
            hdr: Hdr::new(),
            acc: 0.0,
            rate: 20.0,
            generation: 0,
            pop: 0,
            stale: 0,
            rng: Rng::new(seed_from_time()),
        }
    }

    fn seed(&mut self) {
        for c in self.cells.iter_mut() {
            *c = if self.rng.f64() < 0.22 { 1 } else { 0 };
        }
        self.generation = 0;
        self.stale = 0;
        self.pop = 0;
    }

    fn step(&mut self) {
        let (w, h) = (self.w, self.h);
        if w == 0 || h == 0 {
            return;
        }
        let mut pop = 0usize;
        {
            let cells = &self.cells;
            let next = &mut self.next;
            for y in 0..h {
                let (ym, yp) = ((y + h - 1) % h, (y + 1) % h);
                for x in 0..w {
                    let (xm, xp) = ((x + w - 1) % w, (x + 1) % w);
                    let n = alive(cells, w, xm, ym)
                        + alive(cells, w, x, ym)
                        + alive(cells, w, xp, ym)
                        + alive(cells, w, xm, y)
                        + alive(cells, w, xp, y)
                        + alive(cells, w, xm, yp)
                        + alive(cells, w, x, yp)
                        + alive(cells, w, xp, yp);
                    let age = cells[y * w + x];
                    let new_age = match (age > 0, n) {
                        (true, 2) | (true, 3) => age.saturating_add(1),
                        (false, 3) => 1,
                        _ => 0,
                    };
                    if new_age > 0 {
                        pop += 1;
                    }
                    next[y * w + x] = new_age;
                }
            }
        }
        std::mem::swap(&mut self.cells, &mut self.next);
        self.generation += 1;
        if pop == self.pop {
            self.stale += 1;
        } else {
            self.stale = 0;
        }
        self.pop = pop;
        if self.stale > 60 {
            self.seed();
        }
    }
}

impl Scene for Life {
    fn name(&self) -> &'static str {
        "life"
    }

    fn status(&self) -> String {
        format!(
            "Conway's Life | gen {} pop {} rate {:.0}/s | +/-: speed  x: reseed  g: glider",
            self.generation, self.pop, self.rate
        )
    }

    fn resize(&mut self, px_w: usize, px_h: usize) {
        self.w = px_w;
        self.h = px_h;
        self.cells = vec![0; px_w * px_h];
        self.next = vec![0; px_w * px_h];
        self.hdr.resize(px_w, px_h);
        self.seed();
    }

    fn reset(&mut self) {
        self.seed();
    }

    fn update(&mut self, dt: f64) {
        self.acc = (self.acc + dt * self.rate).min(10.0);
        while self.acc >= 1.0 {
            self.step();
            self.acc -= 1.0;
        }
        self.hdr.fade(0.78f32.powf(dt as f32 * 30.0));
        for y in 0..self.h {
            for x in 0..self.w {
                let a = self.cells[y * self.w + x];
                if a > 0 {
                    let hue = 190.0 + (a as f64).min(100.0) * 1.6;
                    let amount = if a == 1 { 1.6 } else { 0.9 };
                    self.hdr.add_px(x, y, lin(hsv(hue, 0.8, 1.0), amount));
                }
            }
        }
    }

    fn draw(&self, canvas: &mut Canvas) {
        self.hdr.blit(canvas, 1.2, 0.5);
    }

    fn key(&mut self, key: Key) {
        match key {
            Key::Char('+') | Key::Char('=') => self.rate = (self.rate * 1.3).min(240.0),
            Key::Char('-') | Key::Char('_') => self.rate = (self.rate / 1.3).max(2.0),
            Key::Char('x') => self.seed(),
            Key::Char('g') => {
                if self.w > 0 && self.h > 0 {
                    let ox = (self.rng.f64() * self.w as f64) as usize;
                    let oy = (self.rng.f64() * self.h as f64) as usize;
                    for (dx, dy) in GLIDER {
                        let x = (ox + dx) % self.w;
                        let y = (oy + dy) % self.h;
                        self.cells[y * self.w + x] = 1;
                    }
                }
            }
            _ => {}
        }
    }
}
