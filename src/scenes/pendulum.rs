use crate::canvas::Canvas;
use crate::color::hsv;
use crate::hdr::{lin, Hdr};
use crate::rng::{seed_from_time, Rng};
use crate::scene::{Key, Scene};

const G: f64 = 9.81;
const H: f64 = 0.002;
const EPS: f64 = 1e-3;

type State = [f64; 4];

fn deriv(s: State) -> State {
    let (t1, w1, t2, w2) = (s[0], s[1], s[2], s[3]);
    let d = t1 - t2;
    let den = 3.0 - (2.0 * d).cos();
    let a1 = (-3.0 * G * t1.sin()
        - G * (t1 - 2.0 * t2).sin()
        - 2.0 * d.sin() * (w2 * w2 + w1 * w1 * d.cos()))
        / den;
    let a2 = (2.0 * d.sin() * (2.0 * w1 * w1 + 2.0 * G * t1.cos() + w2 * w2 * d.cos())) / den;
    [w1, a1, w2, a2]
}

fn rk4_step(s: State, h: f64) -> State {
    let add = |a: State, b: State, k: f64| -> State {
        [
            a[0] + b[0] * k,
            a[1] + b[1] * k,
            a[2] + b[2] * k,
            a[3] + b[3] * k,
        ]
    };
    let k1 = deriv(s);
    let k2 = deriv(add(s, k1, h * 0.5));
    let k3 = deriv(add(s, k2, h * 0.5));
    let k4 = deriv(add(s, k3, h));
    let mut out = s;
    for i in 0..4 {
        out[i] += (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]) * h / 6.0;
    }
    out
}

struct Pend {
    s: State,
    last: Option<(f32, f32)>,
}

impl Pend {
    fn new(s: State) -> Self {
        Pend { s, last: None }
    }

    fn elbow(&self) -> (f64, f64) {
        (self.s[0].sin(), self.s[0].cos())
    }

    fn tip(&self) -> (f64, f64) {
        (
            self.s[0].sin() + self.s[2].sin(),
            self.s[0].cos() + self.s[2].cos(),
        )
    }
}

pub struct Pendulum {
    a: Pend,
    b: Pend,
    hdr: Hdr,
    acc: f64,
    speed: f64,
    time: f64,
    w: usize,
    h: usize,
    rng: Rng,
}

impl Pendulum {
    pub fn new() -> Self {
        let mut p = Pendulum {
            a: Pend::new([0.0; 4]),
            b: Pend::new([0.0; 4]),
            hdr: Hdr::new(),
            acc: 0.0,
            speed: 1.0,
            time: 0.0,
            w: 0,
            h: 0,
            rng: Rng::new(seed_from_time()),
        };
        p.start(2.0, 2.4);
        p
    }

    fn start(&mut self, t1: f64, t2: f64) {
        self.a = Pend::new([t1, 0.0, t2, 0.0]);
        self.b = Pend::new([t1 + EPS, 0.0, t2, 0.0]);
        self.acc = 0.0;
        self.hdr.clear();
    }

    fn view(&self) -> (f64, f64, f64) {
        let k = 0.46 * self.w.min(self.h) as f64 / 2.0;
        (k, self.w as f64 * 0.5, self.h as f64 * 0.5)
    }

    fn draw_rods(&self, canvas: &mut Canvas, p: &Pend, hue: f64) {
        let (k, cx, cy) = self.view();
        let map = |q: (f64, f64)| -> (i32, i32) {
            ((cx + q.0 * k).round() as i32, (cy + q.1 * k).round() as i32)
        };
        let rod = hsv(hue, 0.25, 1.0);
        let (px, py) = map((0.0, 0.0));
        let (ex, ey) = map(p.elbow());
        let (tx, ty) = map(p.tip());
        canvas.line(px, py, ex, ey, rod);
        canvas.line(ex, ey, tx, ty, rod);
    }
}

impl Scene for Pendulum {
    fn name(&self) -> &'static str {
        "pendulum"
    }

    fn status(&self) -> String {
        let div = (self.a.s[0] - self.b.s[0]).abs() + (self.a.s[2] - self.b.s[2]).abs();
        format!(
            "Double pendulum | twin offset {} rad  divergence {:.3} rad | +/-: speed  x: new start",
            EPS, div
        )
    }

    fn resize(&mut self, px_w: usize, px_h: usize) {
        self.w = px_w;
        self.h = px_h;
        self.hdr.resize(px_w, px_h);
        self.a.last = None;
        self.b.last = None;
    }

    fn reset(&mut self) {
        self.start(2.0, 2.4);
    }

    fn update(&mut self, dt: f64) {
        self.acc = (self.acc + dt * self.speed).min(0.2);
        while self.acc >= H {
            self.a.s = rk4_step(self.a.s, H);
            self.b.s = rk4_step(self.b.s, H);
            self.acc -= H;
        }
        self.time += dt;

        if self.w == 0 || self.h == 0 {
            return;
        }
        self.hdr.fade(0.975f32.powf(dt as f32 * 30.0));
        let (k, cx, cy) = self.view();
        let tips = [(self.a.tip(), 30.0), (self.b.tip(), 190.0)];
        for (i, (tip, base_hue)) in tips.iter().enumerate() {
            let pos = ((cx + tip.0 * k) as f32, (cy + tip.1 * k) as f32);
            let color = hsv(base_hue + self.time * 18.0, 0.8, 1.0);
            let pend = if i == 0 { &mut self.a } else { &mut self.b };
            if let Some((px, py)) = pend.last {
                self.hdr.line(px, py, pos.0, pos.1, lin(color, 0.9));
            }
            pend.last = Some(pos);
        }
    }

    fn draw(&self, canvas: &mut Canvas) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        self.hdr.blit(canvas, 1.0, 0.9);
        self.draw_rods(canvas, &self.b, 190.0);
        self.draw_rods(canvas, &self.a, 30.0);
    }

    fn key(&mut self, key: Key) {
        match key {
            Key::Char('+') | Key::Char('=') => self.speed = (self.speed * 1.25).min(6.0),
            Key::Char('-') | Key::Char('_') => self.speed = (self.speed / 1.25).max(0.1),
            Key::Char('x') => {
                let t1 = self.rng.range(1.5, 3.0);
                let t2 = self.rng.range(1.5, 3.0);
                self.start(t1, t2);
            }
            _ => {}
        }
    }
}
