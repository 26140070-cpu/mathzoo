use crate::canvas::Canvas;
use crate::color::{hsv, WHITE};
use crate::math::{rk4, V3};
use crate::scene::{Key, Scene};
use std::collections::VecDeque;

const MAX_PTS: usize = 5000;
const SUB: usize = 3;
const BURN_IN: usize = 3000;
const PREFILL: usize = 1500;

#[derive(Clone, Copy)]
enum Kind {
    Lorenz,
    Rossler,
    Aizawa,
    Thomas,
    Halvorsen,
}

const KINDS: [Kind; 5] = [
    Kind::Lorenz,
    Kind::Rossler,
    Kind::Aizawa,
    Kind::Thomas,
    Kind::Halvorsen,
];

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Lorenz => "Lorenz",
            Kind::Rossler => "Rossler",
            Kind::Aizawa => "Aizawa",
            Kind::Thomas => "Thomas",
            Kind::Halvorsen => "Halvorsen",
        }
    }

    fn start(self) -> V3 {
        match self {
            Kind::Lorenz => V3::new(0.1, 0.0, 0.0),
            Kind::Rossler => V3::new(1.0, 1.0, 0.0),
            Kind::Aizawa => V3::new(0.1, 0.0, 0.0),
            Kind::Thomas => V3::new(1.1, 1.1, -0.01),
            Kind::Halvorsen => V3::new(-1.48, -1.51, 2.04),
        }
    }

    fn dt(self) -> f64 {
        match self {
            Kind::Lorenz => 0.005,
            Kind::Rossler => 0.02,
            Kind::Aizawa => 0.01,
            Kind::Thomas => 0.05,
            Kind::Halvorsen => 0.005,
        }
    }

    fn deriv(self, p: V3) -> V3 {
        match self {
            Kind::Lorenz => V3::new(
                10.0 * (p.y - p.x),
                p.x * (28.0 - p.z) - p.y,
                p.x * p.y - 8.0 / 3.0 * p.z,
            ),
            Kind::Rossler => V3::new(-p.y - p.z, p.x + 0.2 * p.y, 0.2 + p.z * (p.x - 5.7)),
            Kind::Aizawa => {
                let (a, b, c, d, e, f) = (0.95, 0.7, 0.6, 3.5, 0.25, 0.1);
                V3::new(
                    (p.z - b) * p.x - d * p.y,
                    d * p.x + (p.z - b) * p.y,
                    c + a * p.z - p.z.powi(3) / 3.0 - (p.x * p.x + p.y * p.y) * (1.0 + e * p.z)
                        + f * p.z * p.x.powi(3),
                )
            }
            Kind::Thomas => {
                let b = 0.208186;
                V3::new(
                    p.y.sin() - b * p.x,
                    p.z.sin() - b * p.y,
                    p.x.sin() - b * p.z,
                )
            }
            Kind::Halvorsen => {
                let a = 1.89;
                V3::new(
                    -a * p.x - 4.0 * p.y - 4.0 * p.z - p.y * p.y,
                    -a * p.y - 4.0 * p.z - 4.0 * p.x - p.z * p.z,
                    -a * p.z - 4.0 * p.x - 4.0 * p.y - p.x * p.x,
                )
            }
        }
    }
}

fn burn_in(kind: Kind) -> V3 {
    let f = |p: V3| kind.deriv(p);
    let mut p = kind.start();
    for _ in 0..BURN_IN {
        p = rk4(&f, p, kind.dt());
    }
    p
}

struct Trail {
    head: V3,
    pts: VecDeque<V3>,
}

impl Trail {
    fn new(head: V3) -> Self {
        Trail {
            head,
            pts: VecDeque::new(),
        }
    }

    fn advance(&mut self, kind: Kind) {
        let f = |p: V3| kind.deriv(p);
        let h = kind.dt();
        for _ in 0..SUB {
            self.head = rk4(&f, self.head, h);
        }
        if !(self.head.x.is_finite() && self.head.y.is_finite() && self.head.z.is_finite()) {
            self.head = kind.start();
            self.pts.clear();
        }
        self.pts.push_back(self.head);
        while self.pts.len() > MAX_PTS {
            self.pts.pop_front();
        }
    }
}

pub struct Attractors {
    idx: usize,
    main: Trail,
    twin: Option<Trail>,
    yaw: f64,
    pitch: f64,
    spin: f64,
    speed: f64,
    acc: f64,
    fit_c: V3,
    fit_r: f64,
    w: usize,
    h: usize,
}

impl Attractors {
    pub fn new() -> Self {
        let mut a = Attractors {
            idx: 0,
            main: Trail::new(V3::default()),
            twin: None,
            yaw: 0.0,
            pitch: 0.35,
            spin: 0.35,
            speed: 1.0,
            acc: 0.0,
            fit_c: V3::default(),
            fit_r: 1.0,
            w: 0,
            h: 0,
        };
        a.load();
        a
    }

    fn kind(&self) -> Kind {
        KINDS[self.idx]
    }

    fn load(&mut self) {
        let kind = self.kind();
        self.main = Trail::new(burn_in(kind));
        for _ in 0..PREFILL {
            self.main.advance(kind);
        }
        self.twin = None;
        self.refit(true);
    }

    fn refit(&mut self, snap: bool) {
        if self.main.pts.is_empty() {
            return;
        }
        let mut lo = V3::new(f64::MAX, f64::MAX, f64::MAX);
        let mut hi = V3::new(f64::MIN, f64::MIN, f64::MIN);
        for p in self.main.pts.iter() {
            lo = V3::new(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z));
            hi = V3::new(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z));
        }
        let c = (lo + hi) * 0.5;
        let mut r: f64 = 1e-9;
        for p in self.main.pts.iter() {
            r = r.max((*p - c).norm());
        }
        if snap {
            self.fit_c = c;
            self.fit_r = r;
        } else {
            self.fit_c = self.fit_c + (c - self.fit_c) * 0.05;
            self.fit_r += (r - self.fit_r) * 0.05;
        }
    }

    fn project(&self, p: V3, k: f64) -> (i32, i32) {
        let d = p - self.fit_c;
        let q = V3::new(d.x, d.z, d.y).rot_y(self.yaw).rot_x(self.pitch);
        let cx = self.w as f64 * 0.5;
        let cy = self.h as f64 * 0.5;
        ((cx + q.x * k).round() as i32, (cy - q.y * k).round() as i32)
    }

    fn draw_trail(&self, canvas: &mut Canvas, trail: &Trail, k: f64, hue_old: f64, hue_new: f64) {
        let n = trail.pts.len();
        let mut prev: Option<(i32, i32)> = None;
        for (i, p) in trail.pts.iter().enumerate() {
            let t = i as f64 / n.max(1) as f64;
            let (x, y) = self.project(*p, k);
            let color = hsv(hue_old + (hue_new - hue_old) * t, 0.8, 0.25 + 0.75 * t);
            match prev {
                Some((px, py)) => canvas.line(px, py, x, y, color),
                None => canvas.set(x, y, color),
            }
            prev = Some((x, y));
        }
    }
}

impl Scene for Attractors {
    fn name(&self) -> &'static str {
        "attractors"
    }

    fn status(&self) -> String {
        let mut s = format!(
            "{} | arrows: view  +/-: speed  c: next system  t: twin (butterfly effect)",
            self.kind().name()
        );
        if let Some(t) = &self.twin {
            let d = (self.main.head - t.head).norm();
            s.push_str(&format!(" | twin distance {:.4}", d));
        }
        s
    }

    fn resize(&mut self, px_w: usize, px_h: usize) {
        self.w = px_w;
        self.h = px_h;
    }

    fn reset(&mut self) {
        self.load();
    }

    fn update(&mut self, dt: f64) {
        self.yaw += self.spin * dt;
        self.acc = (self.acc + dt * 60.0 * self.speed).min(30.0);
        let kind = self.kind();
        while self.acc >= 1.0 {
            self.main.advance(kind);
            if let Some(t) = self.twin.as_mut() {
                t.advance(kind);
            }
            self.acc -= 1.0;
        }
        self.refit(false);
    }

    fn draw(&self, canvas: &mut Canvas) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        let k = 0.46 * self.w.min(self.h) as f64 / self.fit_r.max(1e-9);
        self.draw_trail(canvas, &self.main, k, 200.0, 330.0);
        if let Some(t) = &self.twin {
            self.draw_trail(canvas, t, k, 20.0, 60.0);
        }
        let (hx, hy) = self.project(self.main.head, k);
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            canvas.set(hx + dx, hy + dy, WHITE);
        }
        if let Some(t) = &self.twin {
            let (tx, ty) = self.project(t.head, k);
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                canvas.set(tx + dx, ty + dy, hsv(0.0, 0.9, 1.0));
            }
        }
    }

    fn key(&mut self, key: Key) {
        match key {
            Key::Left => self.spin -= 0.15,
            Key::Right => self.spin += 0.15,
            Key::Up => self.pitch = (self.pitch + 0.1).min(1.5),
            Key::Down => self.pitch = (self.pitch - 0.1).max(-1.5),
            Key::Char('+') | Key::Char('=') => self.speed = (self.speed * 1.25).min(8.0),
            Key::Char('-') | Key::Char('_') => self.speed = (self.speed / 1.25).max(0.1),
            Key::Char('c') => {
                self.idx = (self.idx + 1) % KINDS.len();
                self.load();
            }
            Key::Char('t') => {
                if self.twin.is_some() {
                    self.twin = None;
                } else {
                    let mut head = self.main.head;
                    head.x += 1e-3;
                    self.twin = Some(Trail::new(head));
                }
            }
            _ => {}
        }
    }
}
