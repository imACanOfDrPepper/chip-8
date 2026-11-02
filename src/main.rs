mod cpu;

use crate::cpu::Cpu;
use clap::{Arg, Command};
use minifb::{Key, Scale, Window, WindowOptions};
use std::fs;

fn main() {
    let mut args = Command::new("chip-8")
        .args([Arg::new("rom").required(true).value_parser(|path: &str| {
            let metadata = fs::metadata(path).unwrap();
            if !metadata.is_file() {
                Err("Must be a valid path to a file")
            } else if metadata.len() > 3584 {
                Err("ROM is too large")
            } else {
                fs::read(path).map_err(|_| "")
            }
        })])
        .get_matches();

    let rom = args.remove_one::<Vec<u8>>("rom").unwrap();

    let mut cpu = Cpu::new();
    cpu.load_memory(&rom);

    let mut window = Window::new(
        "CHIP-8 Emulator",
        64,
        32,
        WindowOptions {
            scale: Scale::X8,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut buffer = vec![0u32; 64 * 32];

        cpu.cycle();

        for (y, &row) in cpu.frame_buffer.iter().enumerate() {
            for x in 0..64 {
                buffer[64 * y + x] = if (row >> (63 - x)) & 1 == 1 {
                    0xFFFFFF
                } else {
                    0x000000
                };
            }
        }

        window.update_with_buffer(&buffer, 64, 32).unwrap();
    }
}

#[cfg(test)]
mod tests;
