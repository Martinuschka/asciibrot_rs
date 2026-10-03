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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

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

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
    let _ = io::stdout().flush();
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
                eprintln!("Unbekannte Option: {other}\n\n                    Nutzung: mandelbrot [--width N] [--height N] [--iter N] \
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

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut current_zoom = zoom;

    loop {
        // Clear the Ctrl-C flag at the start of each loop iteration
        running.store(true, Ordering::SeqCst);

        // Ask user for rate
        print!("\nEnter zoom rate (frames per second, or 'q' to quit): ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");

        let input = input.trim();
        if input.eq_ignore_ascii_case("q") || input.eq_ignore_ascii_case("quit") {
            break;
        }

        let rate: f64 = match input.parse() {
            Ok(rate) if rate > 0.0 => rate,
            _ => {
                println!("Invalid rate. Please enter a positive number or 'q' to quit.");
                continue;
            }
        };

        let frame_duration = Duration::from_secs_f64(1.0 / rate);
        let zoom_factor = 0.9; // Zoom in by 10% each frame
        let start_zoom = current_zoom;

        // Main zoom loop
        while running.load(Ordering::SeqCst) {
            let start = Instant::now();

            // Clear screen and render
            clear_screen();
            let output = render(width, height, max_iter, center_r, center_i, current_zoom);
            let stdout = io::stdout();
            let mut out = stdout.lock();
            let _ = out.write_all(output.as_bytes());
            let _ = out.flush();

            // Update zoom for next frame
            current_zoom *= zoom_factor;

            // Sleep for the remaining frame time
            let elapsed = start.elapsed();
            if elapsed < frame_duration {
                thread::sleep(frame_duration - elapsed);
            }
        }

        // Reset zoom to initial value when interrupted
        current_zoom = start_zoom;
    }

    println!("Goodbye!");
}
