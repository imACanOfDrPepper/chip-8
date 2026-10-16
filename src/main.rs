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

struct Cpu {
    memory: [u8; 4096],
    registers: [u8; 16],
    index: u16,
    stack: [u16; 32],
    stack_pointer: u8,
    sound_timer: u8,
    delay_timer: u8,
    frame_buffer: [u64; 32],
    pc: u16,
}

impl Cpu {
    fn new() -> Self {
        Cpu {
            memory: [0; 4096],
            registers: [0; 16],
            index: 0,
            stack: [0; 32],
            stack_pointer: 0,
            sound_timer: 0,
            delay_timer: 0,
            frame_buffer: [0; 32],
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
        while let (Some(msb), Some(lsb)) = (
            self.memory.get(self.pc as usize),
            self.memory.get((self.pc + 1) as usize),
        ) {
            match (*msb, *lsb) {
                (0x00, 0xE0) => {
                    self.frame_buffer = [0; 32];
                }, 
                (0x00, 0xEE) => {
                    self.pc = self.stack[self.stack_pointer as usize];
                    self.stack_pointer -= 1;
                },
                (msb, lsb) if ((msb >> 4) & 0xF) == 0x1 => {
                    self.pc = u16::from_be_bytes([msb & 0xF, lsb]);
                },
                (msb, lsb) if ((msb >> 4) & 0xF) == 0x2 => {
                    self.stack_pointer += 1;
                    self.stack[self.stack_pointer as usize] = self.pc;
                    self.pc = u16::from_be_bytes([msb & 0xF, lsb]);
                },
                (msb, lsb) if (msb >> 4) == 0x3 => {
                    if self.registers[(msb & 0xF) as usize] == lsb {
                        self.pc += 2;
                    }
                },
                (msb, lsb) if (msb >> 4) == 0x4 => {
                    if self.registers[(msb & 0xF) as usize] != lsb {
                        self.pc += 2;
                    }
                },
                (msb, lsb) if (msb >> 4) == 0x5 && (lsb & 0xF) == 0 => {
                    if self.registers[(msb & 0xF) as usize] == self.registers[(lsb >> 4) as usize] {
                        self.pc += 2;
                    }
                },
                (msb, lsb) if (msb >> 4) == 0x6 => {
                    self.registers[(msb & 0xF) as usize] = lsb;
                },
                (msb, lsb) if (msb >> 4) == 0x7 => {
                    self.registers[(msb & 0xF) as usize] += lsb;
                },
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 0 => {
                    self.registers[(msb & 0xF) as usize] = self.registers[(lsb >> 4) as usize];
                },
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 1 => {
                    self.registers[(msb & 0xF) as usize] |= self.registers[(lsb >> 4) as usize]
                },
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 2 => {
                    self.registers[(msb & 0xF) as usize] &= self.registers[(lsb >> 4) as usize]
                },
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 3 => {
                    self.registers[(msb & 0xF) as usize] ^= self.registers[(lsb >> 4) as usize]
                },
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 4 => {
                    let (result, carry) = self.registers[(msb & 0xF) as usize]
                        .overflowing_add(self.registers[(lsb >> 4) as usize]);
                    self.registers[(msb & 0xF) as usize] = result;
                    self.registers[0xF] = carry as u8;
                },
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 5 => {
                    let (result, carry) = self.registers[(msb & 0xF) as usize]
                        .overflowing_sub(self.registers[(lsb >> 4) as usize]);
                    self.registers[(msb & 0xF) as usize] = result;
                    self.registers[0xF] = !carry as u8;
                },
                _ => todo!()
            }
        }
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
