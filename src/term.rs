use crate::canvas::Canvas;
use crate::color::Rgb;
use crossterm::{cursor, execute, terminal};
use std::fmt::Write as _;
use std::io::{self, Stdout, Write};

pub struct Term {
    out: Stdout,
    buf: String,
}

impl Term {
    pub fn enter() -> io::Result<Term> {
        let mut out = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(
            out,
            terminal::EnterAlternateScreen,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All)
        )?;
        Ok(Term {
            out,
            buf: String::new(),
        })
    }

    pub fn size() -> (usize, usize) {
        let (w, h) = terminal::size().unwrap_or((80, 24));
        (w as usize, h as usize)
    }

    pub fn draw(&mut self, canvas: &Canvas, hud: &str) -> io::Result<()> {
        self.buf.clear();
        let mut last: Option<Rgb> = None;
        for row in 0..canvas.rows() {
            let _ = write!(self.buf, "\x1b[{};1H", row + 1);
            for col in 0..canvas.cols() {
                match canvas.cell(col, row) {
                    Some((ch, c)) => {
                        if last != Some(c) {
                            let _ = write!(self.buf, "\x1b[38;2;{};{};{}m", c.0, c.1, c.2);
                            last = Some(c);
                        }
                        self.buf.push(ch);
                    }
                    None => self.buf.push(' '),
                }
            }
        }
        let width = canvas.cols();
        let _ = write!(self.buf, "\x1b[{};1H\x1b[0m\x1b[7m", canvas.rows() + 1);
        let text: String = hud.chars().take(width).collect();
        self.buf.push_str(&text);
        for _ in text.chars().count()..width {
            self.buf.push(' ');
        }
        self.buf.push_str("\x1b[0m");
        self.out.write_all(self.buf.as_bytes())?;
        self.out.flush()
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        let _ = execute!(self.out, cursor::Show, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
