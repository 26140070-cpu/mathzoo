# mathzoo

Explore mathematical worlds in your terminal. `mathzoo` is a colorful, real-time collection of generative art and interactive simulations, rendered with true-color Braille characters for up to eight visual samples per terminal cell.

## Install

```sh
cargo install mathzoo
```

## Run

```sh
mathzoo
mathzoo reaction
mathzoo mand
mathzoo --list
```

Scene names accept prefixes. Use `mathzoo --help` to see command-line options.

## Scenes

| Scene | Description | Scene controls |
| --- | --- | --- |
| `attractors` | Rotating Lorenz, Rossler, Aizawa, Thomas, and Halvorsen strange attractors, integrated with RK4 | Arrows rotate, `+`/`-` change speed, `c` changes system, `t` toggles a nearby twin |
| `boids` | A glowing flock following cohesion, alignment, and separation rules | Arrows tune flocking, `+`/`-` change flock size |
| `life` | Conway's Game of Life with glowing trails and age-based color | `g` adds a glider, `x` reseeds, `+`/`-` change simulation speed |
| `logistic` | A color-density bifurcation diagram with an interactive Lyapunov-exponent cursor | Left/right move the cursor, `+`/`-` zoom |
| `mandelbrot` | The Mandelbrot set and a continuously evolving Julia set | Arrows pan, `+`/`-` zoom, `j` switches set |
| `pendulum` | A double pendulum with two nearly identical starting states | `x` randomizes the start, `+`/`-` change speed |
| `reaction` | Gray-Scott reaction-diffusion patterns, from coral-like forms to mazes | `g` cycles seed shapes, `p` changes chemistry, `c` changes palette, `x` adds seeds, `+`/`-` change speed |

The reaction scene includes five seed layouts (bloom, ring, grid, spiral, and scatter) and twelve color palettes. Other scenes also use animated color, trails, and bloom effects.

## Global controls

`Tab`/`n` selects the next scene, `b` selects the previous scene, `1`-`9` jumps to a scene, `Space` pauses, `r` resets, `h` toggles help, and `q` or `Esc` exits. Scene-specific controls are also shown in the status bar.

## Rendering

The app uses ANSI true color and Unicode Braille characters. For the intended display, use a terminal that supports both. The project has no runtime assets and uses `crossterm` for terminal input and raw mode.

## License

MIT
