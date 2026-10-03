# asciibrot_rs

A tiny Rust program that renders the Mandelbrot set as ASCII art directly in the terminal.

It is intentionally dependency-free and runs with plain `cargo run` or `rustc`.

## Features

- Draws the Mandelbrot set in terminal output
- Adjustable image size, iteration count, zoom, and center point
- No external crates required
- Fast enough for local exploration in a terminal

## Requirements

- Rust toolchain (`rustc` and `cargo`)

## Build and run

```bash
cargo run --release
```

Optional arguments:

```bash
cargo run --release -- --width 120 --height 45 --iter 120
cargo run --release -- --center -0.7441,0.0005 --zoom 0.005
```

## Options

- `--width`, `-w`: output width in characters
- `--height`, `-h`: output height in characters
- `--iter`, `-i`: maximum Mandelbrot iterations
- `--center`, `-c`: center point as `real,imag` (for example `-0.5,0`)
- `--zoom`, `-z`: zoom level

## Vista

Exciting coordinates to try out (show the famous Seahorse Valley):

```bash
cargo run --release -- --center -0.7441,0.0005 --zoom 0.005 --iter 200
```
