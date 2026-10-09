use crate::color::Rgb;

const DOT_BITS: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];
const MAX_LINE: i64 = 8192;

pub struct Canvas {
    cols: usize,
    rows: usize,
    bits: Vec<u8>,
    colors: Vec<Rgb>,
}

impl Canvas {
    pub fn new(cols: usize, rows: usize) -> Self {
        let n = cols * rows;
        Canvas {
            cols,
            rows,
            bits: vec![0; n],
            colors: vec![Rgb(255, 255, 255); n],
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        if cols != self.cols || rows != self.rows {
            *self = Canvas::new(cols, rows);
        }
    }

    pub fn clear(&mut self) {
        for b in self.bits.iter_mut() {
            *b = 0;
        }
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn px_w(&self) -> usize {
        self.cols * 2
    }

    pub fn px_h(&self) -> usize {
        self.rows * 4
    }

    pub fn set(&mut self, x: i32, y: i32, color: Rgb) {
        if x < 0 || y < 0 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.px_w() || y >= self.px_h() {
            return;
        }
        let idx = (y / 4) * self.cols + x / 2;
        self.bits[idx] |= DOT_BITS[x % 2][y % 4];
        self.colors[idx] = color;
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgb) {
        let (x0, y0, x1, y1) = (x0 as i64, y0 as i64, x1 as i64, y1 as i64);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        if dx > MAX_LINE || -dy > MAX_LINE {
            return;
        }
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let (mut x, mut y) = (x0, y0);
        loop {
            self.set(x as i32, y as i32, color);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn cell(&self, col: usize, row: usize) -> Option<(char, Rgb)> {
        let i = row * self.cols + col;
        let b = self.bits[i];
        if b == 0 {
            return None;
        }
        char::from_u32(0x2800 + b as u32).map(|ch| (ch, self.colors[i]))
    }
}
