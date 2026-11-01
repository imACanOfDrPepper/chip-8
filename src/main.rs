mod cpu;

use crate::cpu::Cpu;
use clap::{Arg, Command};
use crossterm::{QueueableCommand, cursor::{self, Hide}, execute, style::Print, terminal::{Clear, ClearType}};
use std::{
    fs,
    io::{self, Write},
};

fn main() -> io::Result<()> {
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

    let mut stdout = io::stdout();

    // Temp
    cpu.frame_buffer[0] = 0xF0;
    cpu.frame_buffer[1] = 0x90;
    cpu.frame_buffer[2] = 0x90;
    cpu.frame_buffer[3] = 0x90;
    cpu.frame_buffer[4] = 0xF0;

    let _ = execute!(
        stdout,
        Hide,
        Clear(ClearType::All)
    );

    loop {
        for row in 0..32 {
            for bit in 0..64 {
                let pixel = (cpu.frame_buffer[row] >> (63 - bit)) & 1;

                stdout.queue(cursor::MoveTo(bit as u16, row as u16))?;

                if pixel == 1 {
                    stdout.queue(Print("█"))?;
                } else {
                    stdout.queue(Print(" "))?;
                }
            }
        }

        stdout.flush()?;
    }
}

#[cfg(test)]
mod tests;
