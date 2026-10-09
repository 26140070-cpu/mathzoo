use crate::canvas::Canvas;
use crate::color::hsv;
use crate::hdr::{lin, Hdr};
use crate::rng::{seed_from_time, Rng};
use crate::scene::{Key, Scene};
use std::f64::consts::TAU;

struct Boid {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
}

pub struct Boids {
    w: usize,
    h: usize,
    flock: Vec<Boid>,
    hdr: Hdr,
    rng: Rng,
    count: usize,
    cohesion: f64,
    separation: f64,
}

impl Boids {
    pub fn new() -> Self {
        Boids {
            w: 0,
            h: 0,
            flock: Vec::new(),
            hdr: Hdr::new(),
            rng: Rng::new(seed_from_time()),
            count: 220,
            cohesion: 1.0,
            separation: 1.0,
        }
    }

    fn max_speed(&self) -> f64 {
        0.35 * self.w.min(self.h) as f64
    }

    fn spawn(&mut self) {
        self.flock.clear();
        let (w, h) = (self.w.max(1) as f64, self.h.max(1) as f64);
        let speed = self.max_speed() * 0.7;
        for _ in 0..self.count {
            let a = self.rng.range(0.0, TAU);
            let b = Boid {
                x: self.rng.range(0.0, w),
                y: self.rng.range(0.0, h),
                vx: a.cos() * speed,
                vy: a.sin() * speed,
            };
            self.flock.push(b);
        }
        self.hdr.clear();
    }
}

impl Scene for Boids {
    fn name(&self) -> &'static str {
        "boids"
    }

    fn status(&self) -> String {
        format!(
            "Flocking, {} boids | up/down: cohesion {:.2}  left/right: separation {:.2}  +/-: count",
            self.flock.len(),
            self.cohesion,
            self.separation
        )
    }

    fn resize(&mut self, px_w: usize, px_h: usize) {
        self.w = px_w;
        self.h = px_h;
        self.hdr.resize(px_w, px_h);
        self.spawn();
    }

    fn reset(&mut self) {
        self.spawn();
    }

    fn update(&mut self, dt: f64) {
        let n = self.flock.len();
        if n == 0 || self.w == 0 || self.h == 0 {
            return;
        }
        let (w, h) = (self.w as f64, self.h as f64);
        let vmax = self.max_speed();
        let vmin = vmax * 0.45;
        let vision = 0.12 * w.min(h);
        let sep = vision * 0.4;

        let mut accs = vec![(0.0f64, 0.0f64); n];
        for i in 0..n {
            let a = &self.flock[i];
            let (mut cx, mut cy): (f64, f64) = (0.0, 0.0);
            let (mut avx, mut avy): (f64, f64) = (0.0, 0.0);
            let (mut ax, mut ay): (f64, f64) = (0.0, 0.0);
            let mut cnt: f64 = 0.0;
            for j in 0..n {
                if i == j {
                    continue;
                }
                let b = &self.flock[j];
                let mut dx = b.x - a.x;
                let mut dy = b.y - a.y;
                if dx > w * 0.5 {
                    dx -= w;
                } else if dx < -w * 0.5 {
                    dx += w;
                }
                if dy > h * 0.5 {
                    dy -= h;
                } else if dy < -h * 0.5 {
                    dy += h;
                }
                let d2 = dx * dx + dy * dy;
                if d2 > vision * vision {
                    continue;
                }
                cnt += 1.0;
                cx += dx;
                cy += dy;
                avx += b.vx;
                avy += b.vy;
                if d2 < sep * sep {
                    let d = d2.sqrt().max(1e-6);
                    let push = (1.0 - d / sep) * vmax * 8.0 * self.separation;
                    ax -= dx / d * push;
                    ay -= dy / d * push;
                }
            }
            if cnt > 0.0 {
                ax += cx / cnt * 1.5 * self.cohesion;
                ay += cy / cnt * 1.5 * self.cohesion;
                ax += (avx / cnt - a.vx) * 4.0;
                ay += (avy / cnt - a.vy) * 4.0;
            }
            accs[i] = (ax, ay);
        }

        for (b, &(ax, ay)) in self.flock.iter_mut().zip(accs.iter()) {
            b.vx += ax * dt;
            b.vy += ay * dt;
            let sp = (b.vx * b.vx + b.vy * b.vy).sqrt();
            if sp > vmax {
                b.vx *= vmax / sp;
                b.vy *= vmax / sp;
            } else if sp < 1e-9 {
                b.vx = vmin;
            } else if sp < vmin {
                b.vx *= vmin / sp;
                b.vy *= vmin / sp;
            }
            b.x = (b.x + b.vx * dt).rem_euclid(w);
            b.y = (b.y + b.vy * dt).rem_euclid(h);
        }

        self.hdr.fade(0.86f32.powf(dt as f32 * 30.0));
        for b in &self.flock {
            let color = hsv(b.vy.atan2(b.vx).to_degrees(), 0.75, 1.0);
            let (tx, ty) = (b.x - b.vx * 0.12, b.y - b.vy * 0.12);
            self.hdr.line(
                tx as f32,
                ty as f32,
                b.x as f32,
                b.y as f32,
                lin(color, 0.55),
            );
            self.hdr.splat(b.x as f32, b.y as f32, lin(color, 1.2));
        }
    }

    fn draw(&self, canvas: &mut Canvas) {
        self.hdr.blit(canvas, 1.1, 0.7);
    }

    fn key(&mut self, key: Key) {
        match key {
            Key::Up => self.cohesion = (self.cohesion * 1.2).min(6.0),
            Key::Down => self.cohesion = (self.cohesion / 1.2).max(0.05),
            Key::Right => self.separation = (self.separation * 1.2).min(6.0),
            Key::Left => self.separation = (self.separation / 1.2).max(0.1),
            Key::Char('+') | Key::Char('=') => {
                self.count = (self.count + 20).min(600);
                self.spawn();
            }
            Key::Char('-') | Key::Char('_') => {
                self.count = self.count.saturating_sub(20).max(20);
                self.spawn();
            }
            _ => {}
        }
    }
}
