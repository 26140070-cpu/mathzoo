mod app;
mod canvas;
mod color;
mod hdr;
mod math;
mod rng;
mod scene;
mod scenes;
mod term;

const USAGE: &str = "\
mathzoo - a zoo of mathematical creatures in your terminal

USAGE:
    mathzoo [SCENE]

OPTIONS:
    --list         list the available scenes
    -h, --help     show this help
    -V, --version  show the version

Run with no arguments to start at the first scene. SCENE may be any prefix
of a scene name, e.g. `mathzoo mand`. Press h inside for the key bindings.";

fn main() {
    let mut start = None;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                return;
            }
            "-V" | "--version" => {
                println!("mathzoo {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--list" => {
                for name in scenes::names() {
                    println!("{name}");
                }
                return;
            }
            other => match scenes::find(other) {
                Some(i) => start = Some(i),
                None => {
                    eprintln!("mathzoo: unknown scene '{other}' (try --list)");
                    std::process::exit(2);
                }
            },
        }
    }
    if let Err(e) = app::run(start) {
        eprintln!("mathzoo: {e}");
        std::process::exit(1);
    }
}
