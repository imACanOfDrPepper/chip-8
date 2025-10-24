use clap::{Arg, Command};
use std::fs;

fn main() {
    let mut args = Command::new("chip-8")
        .args([
            Arg::new("rom")
                .required(true)
                .value_parser(|path: &str| {
                    let metadata = fs::metadata(path).unwrap();
                    if !metadata.is_file() {
                        Err("Must be a valid path to a file")
                    } else if metadata.len() > 3584 {
                        Err("ROM is too large")
                    } else {
                        fs::read(path).map_err(|_| "")
                    }
                })
        ])
        .get_matches();

    let rom = args.remove_one::<Vec<u8>>("rom").unwrap();
    
    let mut cpu = Cpu::new();
    cpu.load_memory(&rom);
    
    loop {
        todo!() // implement executing code
    }
}

struct Cpu {
    memory: [u8; 4096],
    registers: [u8; 16],
    index: u16,
    stack: [u8; 64],
    stack_pointer: u8,
    sound_timer: u8,
    delay_timer: u8,
    frame_buffer: [bool; 64*32],
    pc: u16,
}

impl Cpu {
    fn new() -> Self {
        Cpu {
            memory: [0; 4096],
            registers: [0; 16],
            index: 0,
            stack: [0; 64],
            stack_pointer: 0,
            sound_timer: 0,
            delay_timer: 0,
            frame_buffer: [false; 64*32],
            pc: 0x200,
        }
    }
    
    fn load_memory(&mut self, rom: &[u8]) {
        for (i, byte) in FONT_DATA.iter().enumerate() {
            self.memory[i] = *byte;
        }

        for (i, byte) in rom.iter().enumerate() {
            self.memory[0x200 + i] = *byte;
        }
    }

    fn cycle(&mut self) {
        todo!() // implement executing code
    }
}

const FONT_DATA: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

