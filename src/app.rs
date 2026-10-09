use crate::canvas::Canvas;
use crate::scene::{Key, Scene};
use crate::scenes;
use crate::term::Term;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::io;
use std::time::{Duration, Instant};

const FRAME: Duration = Duration::from_millis(40);
const GLOBAL_HELP: &str =
    "Tab/n: next  b: prev  1-7: jump  space: pause  r: reset  h: toggle help  q/Esc: quit";

fn canvas_dims(w: usize, h: usize) -> (usize, usize) {
    (w.max(10), h.saturating_sub(1).max(4))
}

fn ensure_size(scene: &mut Box<dyn Scene>, known: &mut (usize, usize), px: (usize, usize)) {
    if *known != px {
        scene.resize(px.0, px.1);
        *known = px;
    }
}

pub fn run(start: Option<usize>) -> io::Result<()> {
    let mut term = Term::enter()?;
    let mut scenes = scenes::all();
    let n = scenes.len();
    let mut sizes = vec![(0usize, 0usize); n];
    let mut cur = start.unwrap_or(0).min(n - 1);

    let (mut tw, mut th) = Term::size();
    let (c, r) = canvas_dims(tw, th);
    let mut canvas = Canvas::new(c, r);

    let mut paused = false;
    let mut show_help = false;
    let mut last = Instant::now();

    loop {
        let t0 = Instant::now();

        let (nw, nh) = Term::size();
        if (nw, nh) != (tw, th) {
            tw = nw;
            th = nh;
            let (c, r) = canvas_dims(tw, th);
            canvas.resize(c, r);
        }
        let px = (canvas.px_w(), canvas.px_h());
        ensure_size(&mut scenes[cur], &mut sizes[cur], px);

        while event::poll(Duration::ZERO)? {
            if let Event::Key(k) = event::read()? {
                if k.kind == KeyEventKind::Release {
                    continue;
                }
                let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
                match k.code {
                    KeyCode::Char('c') if ctrl => return Ok(()),
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Tab | KeyCode::Char('n') => cur = (cur + 1) % n,
                    KeyCode::BackTab | KeyCode::Char('b') => cur = (cur + n - 1) % n,
                    KeyCode::Char(' ') => paused = !paused,
                    KeyCode::Char('r') => scenes[cur].reset(),
                    KeyCode::Char('h') | KeyCode::Char('?') => show_help = !show_help,
                    KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                        let i = c as usize - '1' as usize;
                        if i < n {
                            cur = i;
                        }
                    }
                    KeyCode::Up => scenes[cur].key(Key::Up),
                    KeyCode::Down => scenes[cur].key(Key::Down),
                    KeyCode::Left => scenes[cur].key(Key::Left),
                    KeyCode::Right => scenes[cur].key(Key::Right),
                    KeyCode::Char(c) => scenes[cur].key(Key::Char(c)),
                    _ => {}
                }
                ensure_size(&mut scenes[cur], &mut sizes[cur], px);
            }
        }

        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64().min(0.1);
        last = now;
        if !paused {
            scenes[cur].update(dt);
        }
        canvas.clear();
        scenes[cur].draw(&mut canvas);

        let hud = if show_help {
            GLOBAL_HELP.to_string()
        } else {
            format!(
                "[{}/{}] {}{} | {}",
                cur + 1,
                n,
                scenes[cur].name(),
                if paused { " (paused)" } else { "" },
                scenes[cur].status()
            )
        };
        term.draw(&canvas, &hud)?;

        let spent = t0.elapsed();
        if spent < FRAME {
            event::poll(FRAME - spent)?;
        }
    }
}
