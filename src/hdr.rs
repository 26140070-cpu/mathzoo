use crate::canvas::Canvas;
use crate::color::Rgb;

const KERNEL: [f32; 5] = [1.0 / 16.0, 4.0 / 16.0, 6.0 / 16.0, 4.0 / 16.0, 1.0 / 16.0];

pub fn lin(c: Rgb, amount: f32) -> [f32; 3] {
    [
        c.0 as f32 / 255.0 * amount,
        c.1 as f32 / 255.0 * amount,
        c.2 as f32 / 255.0 * amount,
    ]
}

fn tone(v: [f32; 3], exposure: f32) -> Rgb {
    let f = |c: f32| ((1.0 - (-c * exposure).exp()).clamp(0.0, 1.0).powf(0.85) * 255.0) as u8;
    Rgb(f(v[0]), f(v[1]), f(v[2]))
}

pub struct Hdr {
    w: usize,
    h: usize,
    data: Vec<[f32; 3]>,
}

impl Hdr {
    pub fn new() -> Self {
        Hdr {
            w: 0,
            h: 0,
            data: Vec::new(),
        }
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.w = w;
        self.h = h;
        self.data = vec![[0.0; 3]; w * h];
    }

    pub fn clear(&mut self) {
        for p in self.data.iter_mut() {
            *p = [0.0; 3];
        }
    }

    pub fn fade(&mut self, k: f32) {
        for p in self.data.iter_mut() {
            p[0] *= k;
            p[1] *= k;
            p[2] *= k;
        }
    }

    pub fn add_px(&mut self, x: usize, y: usize, c: [f32; 3]) {
        if x < self.w && y < self.h {
            let p = &mut self.data[y * self.w + x];
            p[0] += c[0];
            p[1] += c[1];
            p[2] += c[2];
        }
    }

    pub fn splat(&mut self, x: f32, y: f32, c: [f32; 3]) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let (ix, iy) = (x0 as i64, y0 as i64);
        let taps = [
            (0i64, 0i64, (1.0 - fx) * (1.0 - fy)),
            (1, 0, fx * (1.0 - fy)),
            (0, 1, (1.0 - fx) * fy),
            (1, 1, fx * fy),
        ];
        for (dx, dy, wgt) in taps {
            let (px, py) = (ix + dx, iy + dy);
            if px < 0 || py < 0 || px >= self.w as i64 || py >= self.h as i64 {
                continue;
            }
            let p = &mut self.data[py as usize * self.w + px as usize];
            p[0] += c[0] * wgt;
            p[1] += c[1] * wgt;
            p[2] += c[2] * wgt;
        }
    }

    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, c: [f32; 3]) {
        let len = ((x1 - x0) * (x1 - x0) + (y1 - y0) * (y1 - y0)).sqrt();
        if !len.is_finite() || len > 4096.0 {
            return;
        }
        let n = len.ceil().max(1.0) as usize;
        for i in 0..=n {
            let t = i as f32 / n as f32;
            self.splat(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, c);
        }
    }

    fn blur_h(&self) -> Vec<[f32; 3]> {
        let (w, h) = (self.w, self.h);
        let mut out = vec![[0.0f32; 3]; w * h];
        for y in 0..h {
            for x in 0..w {
                let mut acc = [0.0f32; 3];
                for (i, k) in KERNEL.iter().enumerate() {
                    let xx = (x as i64 + i as i64 - 2).clamp(0, w as i64 - 1) as usize;
                    let p = self.data[y * w + xx];
                    acc[0] += p[0] * *k;
                    acc[1] += p[1] * *k;
                    acc[2] += p[2] * *k;
                }
                out[y * w + x] = acc;
            }
        }
        out
    }

    pub fn blit(&self, canvas: &mut Canvas, exposure: f32, glow: f32) {
        let (w, h) = (self.w, self.h);
        if w == 0 || h == 0 {
            return;
        }
        let tmp = if glow > 0.0 {
            self.blur_h()
        } else {
            Vec::new()
        };
        for y in 0..h {
            for x in 0..w {
                let mut v = self.data[y * w + x];
                if glow > 0.0 {
                    let mut acc = [0.0f32; 3];
                    for (i, k) in KERNEL.iter().enumerate() {
                        let yy = (y as i64 + i as i64 - 2).clamp(0, h as i64 - 1) as usize;
                        let p = tmp[yy * w + x];
                        acc[0] += p[0] * *k;
                        acc[1] += p[1] * *k;
                        acc[2] += p[2] * *k;
                    }
                    v[0] += acc[0] * glow;
                    v[1] += acc[1] * glow;
                    v[2] += acc[2] * glow;
                }
                canvas.set(x as i32, y as i32, tone(v, exposure));
            }
        }
    }
}
