mod cpu;

use crate::cpu::Cpu;
use clap::{Arg, Command};
use minifb::{Key, Scale, Window, WindowOptions};
use rodio::{DeviceSinkBuilder, Player, Source, source::SineWave};
use std::{
    fs, thread,
    time::{Duration, Instant},
};

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

    let target_frame_time = Duration::from_secs_f64(1.0 / 60.0);

    let mut stream_handle = DeviceSinkBuilder::open_default_sink().unwrap();
    stream_handle.log_on_drop(false);

    let player = Player::connect_new(&stream_handle.mixer());

    let beep = SineWave::new(440.0).amplify(0.20);

    player.append(beep);
    player.pause();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let frame_start = Instant::now();

        let mut buffer = vec![0u32; 64 * 32];
        let inputs = get_inputs(&window);

        for _ in 0..11 {
            cpu.cycle(inputs);
        }

        cpu.blanked = false;

        if cpu.sound_timer > 0 {
            cpu.sound_timer -= 1;
            if player.is_paused() {
                player.play();
            }
        } else {
            if !player.is_paused() {
                player.pause();
            }
        }

        if cpu.delay_timer > 0 {
            cpu.delay_timer -= 1
        }

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

        let elapsed = frame_start.elapsed();

        if elapsed < target_frame_time {
            thread::sleep(target_frame_time - elapsed);
        }
    }
}

fn get_inputs(window: &Window) -> [bool; 16] {
    [
        window.is_key_down(Key::X),
        window.is_key_down(Key::Key1),
        window.is_key_down(Key::Key2),
        window.is_key_down(Key::Key3),
        window.is_key_down(Key::Q),
        window.is_key_down(Key::W),
        window.is_key_down(Key::E),
        window.is_key_down(Key::A),
        window.is_key_down(Key::S),
        window.is_key_down(Key::D),
        window.is_key_down(Key::Z),
        window.is_key_down(Key::C),
        window.is_key_down(Key::Key4),
        window.is_key_down(Key::R),
        window.is_key_down(Key::F),
        window.is_key_down(Key::V),
    ]
}

#[cfg(test)]
mod tests;
