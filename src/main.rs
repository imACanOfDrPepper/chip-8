mod cpu;

use crate::cpu::Cpu;
use clap::{Arg, Command};
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

    loop {
        todo!() // implement executing code
    }
}

#[cfg(test)]
mod tests;
