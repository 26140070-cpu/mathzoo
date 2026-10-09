use crate::canvas::Canvas;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Char(char),
}

pub trait Scene {
    fn name(&self) -> &'static str;
    fn status(&self) -> String;
    fn resize(&mut self, px_w: usize, px_h: usize);
    fn reset(&mut self);
    fn update(&mut self, dt: f64);
    fn draw(&self, canvas: &mut Canvas);
    fn key(&mut self, key: Key);
}
