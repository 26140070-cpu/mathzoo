use crate::canvas::Canvas;
use crate::color::{palette, scale, PALETTE_COUNT, PALETTE_NAMES};
use crate::rng::{seed_from_time, Rng};
use crate::scene::{Key, Scene};

const PRESETS: [(&str, f32, f32); 5] = [
    ("coral", 0.0545, 0.062),
    ("maze", 0.029, 0.057),
    ("mitosis", 0.0367, 0.0649),
    ("fingerprint", 0.037, 0.060),
    ("spots", 0.030, 0.062),
];

const SEED_PATTERNS: [&str; 5] = ["bloom", "ring", "grid", "spiral", "scatter"];

fn wrap_offset(center: usize, offset: f64, extent: usize) -> usize {
    ((center as f64 + offset).round() as i64).rem_euclid(extent as i64) as usize
}

pub struct Reaction {
    w: usize,
    h: usize,
    u: Vec<f32>,
    v: Vec<f32>,
    nu: Vec<f32>,
    nv: Vec<f32>,
    preset: usize,
    seed_pattern: usize,
    pal: usize,
    phase: f64,
    spf: usize,
    rng: Rng,
}

fn laplacian(
    a: &[f32],
    w: usize,
    x: usize,
    y: usize,
    xm: usize,
    xp: usize,
    ym: usize,
    yp: usize,
) -> f32 {
    -a[y * w + x]
        + 0.2 * (a[y * w + xm] + a[y * w + xp] + a[ym * w + x] + a[yp * w + x])
        + 0.05 * (a[ym * w + xm] + a[ym * w + xp] + a[yp * w + xm] + a[yp * w + xp])
}

impl Reaction {
    pub fn new() -> Self {
        Reaction {
            w: 0,
            h: 0,
            u: Vec::new(),
            v: Vec::new(),
            nu: Vec::new(),
            nv: Vec::new(),
            preset: 0,
            seed_pattern: 0,
            pal: 0,
            phase: 0.0,
            spf: 12,
            rng: Rng::new(seed_from_time()),
        }
    }

    fn drop_seed(&mut self, cx: usize, cy: usize) {
        let (w, h) = (self.w as i64, self.h as i64);
        for dy in -4i64..=4 {
            for dx in -4i64..=4 {
                let x = (cx as i64 + dx).rem_euclid(w) as usize;
                let y = (cy as i64 + dy).rem_euclid(h) as usize;
                let i = y * self.w + x;
                self.u[i] = 0.5;
                self.v[i] = 0.25;
            }
        }
    }

    fn seed(&mut self) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        for i in 0..self.u.len() {
            self.u[i] = 1.0;
            self.v[i] = 0.0;
        }
        let (cx, cy) = (self.w / 2, self.h / 2);
        let radius = self.w.min(self.h) as f64 * 0.22;
        match self.seed_pattern {
            0 => {
                self.drop_seed(cx, cy);
                for i in 0..8 {
                    let angle = std::f64::consts::TAU * i as f64 / 8.0;
                    self.drop_seed(
                        wrap_offset(cx, angle.cos() * radius * 0.65, self.w),
                        wrap_offset(cy, angle.sin() * radius * 0.65, self.h),
                    );
                }
            }
            1 => {
                for i in 0..16 {
                    let angle = std::f64::consts::TAU * i as f64 / 16.0;
                    self.drop_seed(
                        wrap_offset(cx, angle.cos() * radius, self.w),
                        wrap_offset(cy, angle.sin() * radius, self.h),
                    );
                }
            }
            2 => {
                for gy in 1..=3 {
                    for gx in 1..=3 {
                        self.drop_seed(self.w * gx / 4, self.h * gy / 4);
                    }
                }
            }
            3 => {
                for i in 0..18 {
                    let t = i as f64 / 17.0;
                    let angle = t * std::f64::consts::TAU * 3.0;
                    let distance = radius * t;
                    self.drop_seed(
                        wrap_offset(cx, angle.cos() * distance, self.w),
                        wrap_offset(cy, angle.sin() * distance, self.h),
                    );
                }
            }
            _ => {
                let drops = (self.w * self.h / 1500).max(4);
                for _ in 0..drops {
                    self.add_random_drop();
                }
            }
        }
    }

    fn add_random_drop(&mut self) {
        let cx = (self.rng.f64() * self.w as f64) as usize;
        let cy = (self.rng.f64() * self.h as f64) as usize;
        self.drop_seed(cx.min(self.w - 1), cy.min(self.h - 1));
    }

    fn step(&mut self) {
        let (w, h) = (self.w, self.h);
        let (f, k) = (PRESETS[self.preset].1, PRESETS[self.preset].2);
        for y in 0..h {
            let (ym, yp) = ((y + h - 1) % h, (y + 1) % h);
            for x in 0..w {
                let (xm, xp) = ((x + w - 1) % w, (x + 1) % w);
                let i = y * w + x;
                let (u, v) = (self.u[i], self.v[i]);
                let uvv = u * v * v;
                let lu = laplacian(&self.u, w, x, y, xm, xp, ym, yp);
                let lv = laplacian(&self.v, w, x, y, xm, xp, ym, yp);
                self.nu[i] = (u + lu - uvv + f * (1.0 - u)).clamp(0.0, 1.0);
                self.nv[i] = (v + 0.5 * lv + uvv - (k + f) * v).clamp(0.0, 1.0);
            }
        }
        std::mem::swap(&mut self.u, &mut self.nu);
        std::mem::swap(&mut self.v, &mut self.nv);
    }
}

impl Scene for Reaction {
    fn name(&self) -> &'static str {
        "reaction"
    }

    fn status(&self) -> String {
        let (name, f, k) = PRESETS[self.preset];
        format!(
            "Gray-Scott {} F={:.4} k={:.4} | {} seeds | c: {} palette  g: shape  p: chemistry  x: add seeds  +/-: speed",
            name,
            f,
            k,
            SEED_PATTERNS[self.seed_pattern],
            PALETTE_NAMES[self.pal]
        )
    }

    fn resize(&mut self, px_w: usize, px_h: usize) {
        self.w = px_w;
        self.h = px_h;
        let n = px_w * px_h;
        self.u = vec![1.0; n];
        self.v = vec![0.0; n];
        self.nu = vec![1.0; n];
        self.nv = vec![0.0; n];
        self.seed();
    }

    fn reset(&mut self) {
        self.seed();
    }

    fn update(&mut self, dt: f64) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        self.phase += dt * 0.02;
        for _ in 0..self.spf {
            self.step();
        }
    }

    fn draw(&self, canvas: &mut Canvas) {
        for y in 0..self.h {
            for x in 0..self.w {
                let v = self.v[y * self.w + x] as f64;
                let t = (v * 2.6).min(1.0);
                if t > 0.02 {
                    let base = palette(0.35 + t * 0.55 + self.phase, self.pal);
                    canvas.set(x as i32, y as i32, scale(base, t.powf(0.55)));
                }
            }
        }
    }

    fn key(&mut self, key: Key) {
        match key {
            Key::Char('p') => {
                self.preset = (self.preset + 1) % PRESETS.len();
                self.seed();
            }
            Key::Char('c') => self.pal = (self.pal + 1) % PALETTE_COUNT,
            Key::Char('g') => {
                self.seed_pattern = (self.seed_pattern + 1) % SEED_PATTERNS.len();
                self.seed();
            }
            Key::Char('x') => {
                if self.w > 0 && self.h > 0 {
                    for _ in 0..4 {
                        self.add_random_drop();
                    }
                }
            }
            Key::Char('+') | Key::Char('=') => self.spf = (self.spf + 4).min(60),
            Key::Char('-') | Key::Char('_') => self.spf = self.spf.saturating_sub(4).max(2),
            _ => {}
        }
    }
}
