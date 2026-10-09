use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl V3 {
    pub fn new(x: f64, y: f64, z: f64) -> V3 {
        V3 { x, y, z }
    }

    pub fn norm(self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn rot_y(self, a: f64) -> V3 {
        let (s, c) = a.sin_cos();
        V3::new(c * self.x + s * self.z, self.y, -s * self.x + c * self.z)
    }

    pub fn rot_x(self, a: f64) -> V3 {
        let (s, c) = a.sin_cos();
        V3::new(self.x, c * self.y - s * self.z, s * self.y + c * self.z)
    }
}

impl Add for V3 {
    type Output = V3;
    fn add(self, o: V3) -> V3 {
        V3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}

impl Sub for V3 {
    type Output = V3;
    fn sub(self, o: V3) -> V3 {
        V3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}

impl Mul<f64> for V3 {
    type Output = V3;
    fn mul(self, k: f64) -> V3 {
        V3::new(self.x * k, self.y * k, self.z * k)
    }
}

pub fn rk4<F: Fn(V3) -> V3>(f: &F, p: V3, h: f64) -> V3 {
    let k1 = f(p);
    let k2 = f(p + k1 * (h * 0.5));
    let k3 = f(p + k2 * (h * 0.5));
    let k4 = f(p + k3 * h);
    p + (k1 + k2 * 2.0 + k3 * 2.0 + k4) * (h / 6.0)
}
