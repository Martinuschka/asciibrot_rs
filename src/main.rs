// ASCII-Mandelbrot direkt im Terminal.
//
// Kompiliert ohne externe Crates:
//   cargo run --release
//   cargo run --release -- --width 120 --height 45 --iter 120
//   cargo run --release -- --center -0.7441,0.0005 --zoom 0.005
//
// Alternativ ohne Cargo direkt mit rustc:
//   rustc -O mandelbrot.rs -o mandelbrot && ./mandelbrot

use std::env;
use std::io::{self, Write};

// Zeichenskala: wenig Iterationen (weit außen) -> Punkt, viele -> dichte Zeichen
const CHARS: &[u8] = b" .:-=+*#%@";

/// Gibt die Anzahl der Iterationen bis zur Divergenz zurück.
fn mandelbrot_pixel(cr: f64, ci: f64, max_iter: u32) -> u32 {
    let mut zr = 0.0;
    let mut zi = 0.0;
    for n in 0..max_iter {
        // z = z^2 + c
        let (zr2, zi2) = (zr * zr, zi * zi);
        if zr2 + zi2 > 4.0 {
            return n;
        }
        zi = 2.0 * zr * zi + ci;
        zr = zr2 - zi2 + cr;
    }
    max_iter
}

fn render(width: usize, height: usize, max_iter: u32,
          center_r: f64, center_i: f64, zoom: f64) -> String {
    // zoom = Höhe des sichtbaren Ausschnitts in der komplexen Ebene.
    // Der Faktor 2.1 gleicht das breitere Terminal-Zeichen aus.
    let scale_r = zoom * (width as f64 / height as f64) / 2.1;
    let scale_i = zoom / 2.0;

    let mut buf = String::with_capacity(width * height);
    for row in 0..height {
        let ci = center_i + (row as f64 / height as f64 - 0.5) * scale_i * 2.0;
        for col in 0..width {
            let cr = center_r + (col as f64 / width as f64 - 0.5) * scale_r * 2.0;
            let n = mandelbrot_pixel(cr, ci, max_iter);
            // Iterationszahl auf die Zeichenskala mappen
            let idx = (n as f64 / max_iter as f64 * (CHARS.len() - 1) as f64) as usize;
            buf.push(CHARS[idx] as char);
        }
        buf.push('\n');
    }
    buf
}

fn main() {
    // Manuelle Argument-Parsing-Hilfslogik, um ohne externe Crates auszukommen.
    let mut width = 100usize;
    let mut height = 38usize;
    let mut max_iter = 80u32;
    let mut center = String::from("-0.5,0");
    let mut zoom = 3.0f64;

    let args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--width" | "-w" => { i += 1; width = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(width); }
            "--height" | "-h" => { i += 1; height = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(height); }
            "--iter" | "-i" => { i += 1; max_iter = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(max_iter); }
            "--center" | "-c" => { i += 1; center = args.get(i).cloned().unwrap_or(center); }
            "--zoom" | "-z" => { i += 1; zoom = args.get(i).and_then(|v| v.parse().ok()).unwrap_or(zoom); }
            other => {
                eprintln!("Unbekannte Option: {other}\n\
                    Nutzung: mandelbrot [--width N] [--height N] [--iter N] \
                    [--center real,imag] [--zoom F]");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let (center_r, center_i) = match center
        .split(',')
        .map(|v| v.trim().parse::<f64>())
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(v) if v.len() == 2 => (v[0], v[1]),
        _ => {
            eprintln!("Ungültiges Zentrum: {center} (erwartet z. B. -0.5,0)");
            std::process::exit(1);
        }
    };

    let output = render(width, height, max_iter, center_r, center_i, zoom);
    let stdout = io::stdout();
    let mut out = stdout.lock();
    // Ein einzelner write_all statt Zeile für Zeile: deutlich schneller im Terminal.
    let _ = out.write_all(output.as_bytes());
    let _ = out.flush();
}