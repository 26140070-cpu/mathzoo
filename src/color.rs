#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rgb(pub u8, pub u8, pub u8);

pub const WHITE: Rgb = Rgb(245, 245, 245);
pub const PALETTE_NAMES: [&str; 12] = [
    "aurora",
    "ocean",
    "ember",
    "forest",
    "ultraviolet",
    "candy",
    "arctic",
    "sunset",
    "neon",
    "gold",
    "lagoon",
    "monochrome",
];
pub const PALETTE_COUNT: usize = PALETTE_NAMES.len();

const PALETTES: [[Rgb; 3]; PALETTE_COUNT] = [
    [Rgb(8, 20, 60), Rgb(0, 220, 180), Rgb(245, 80, 210)],
    [Rgb(2, 10, 45), Rgb(0, 110, 220), Rgb(100, 245, 255)],
    [Rgb(35, 0, 10), Rgb(235, 45, 20), Rgb(255, 220, 70)],
    [Rgb(5, 30, 16), Rgb(50, 180, 70), Rgb(230, 250, 120)],
    [Rgb(25, 0, 65), Rgb(130, 35, 240), Rgb(255, 80, 200)],
    [Rgb(50, 0, 55), Rgb(255, 55, 170), Rgb(255, 220, 240)],
    [Rgb(0, 28, 55), Rgb(40, 190, 220), Rgb(230, 255, 255)],
    [Rgb(45, 0, 30), Rgb(250, 90, 35), Rgb(255, 235, 120)],
    [Rgb(0, 30, 35), Rgb(0, 255, 150), Rgb(255, 255, 0)],
    [Rgb(35, 18, 0), Rgb(210, 125, 15), Rgb(255, 250, 180)],
    [Rgb(0, 35, 50), Rgb(0, 210, 190), Rgb(130, 140, 255)],
    [Rgb(12, 12, 18), Rgb(125, 130, 145), Rgb(255, 255, 255)],
];

fn to_u8(v: f64) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

pub fn hsv(h: f64, s: f64, v: f64) -> Rgb {
    let h = h.rem_euclid(360.0);
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Rgb(to_u8(r + m), to_u8(g + m), to_u8(b + m))
}

pub fn palette(t: f64, index: usize) -> Rgb {
    let t = t.rem_euclid(1.0);
    let stops = PALETTES[index % PALETTE_COUNT];
    let position = t * 2.0;
    let segment = (position.floor() as usize).min(1);
    let local = position - segment as f64;
    let start = stops[segment];
    let end = stops[segment + 1];
    let blend = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * local).round() as u8;
    Rgb(
        blend(start.0, end.0),
        blend(start.1, end.1),
        blend(start.2, end.2),
    )
}

pub fn scale(color: Rgb, amount: f64) -> Rgb {
    let amount = amount.clamp(0.0, 1.0);
    Rgb(
        to_u8(color.0 as f64 / 255.0 * amount),
        to_u8(color.1 as f64 / 255.0 * amount),
        to_u8(color.2 as f64 / 255.0 * amount),
    )
}
