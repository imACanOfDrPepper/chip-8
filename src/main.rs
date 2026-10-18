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
    v: [u8; 16],
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
            v: [0; 16],
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
        if let (Some(msb), Some(lsb)) = (
            self.memory.get(self.pc as usize),
            self.memory.get((self.pc + 1) as usize),
        ) {
            self.pc += 2;
            self.pc &= 0x0FFF;

            match (*msb, *lsb) {
                (0x00, 0xE0) => {
                    self.frame_buffer = [0; 32];
                }, 
                (0x00, 0xEE) => {
                    if self.stack_pointer == 0 {
                        eprintln!("WARNING: stack underflow");
                    }

                    self.stack_pointer = self.stack_pointer.wrapping_sub(1) & 15;
                    self.pc = self.stack[self.stack_pointer as usize] & 0x0FFF;
                },
                (msb, lsb) if ((msb >> 4) & 0xF) == 0x1 => {
                    self.pc = u16::from_be_bytes([msb & 0xF, lsb]);
                },
                (msb, lsb) if ((msb >> 4) & 0xF) == 0x2 => {
                    if self.stack_pointer == 16 {
                        eprintln!("WARNING: stack overflow");
                    }

                    self.stack[self.stack_pointer as usize] = self.pc;
                    self.pc = u16::from_be_bytes([msb & 0xF, lsb]);
                    self.stack_pointer = self.stack_pointer.wrapping_add(1) & 15;
                },
                (msb, lsb) if (msb >> 4) == 0x3 => {
                    if self.v[(msb & 0xF) as usize] == lsb {
                        self.pc += 2;
                        self.pc &= 0x0FFF;
                    }
                },
                (msb, lsb) if (msb >> 4) == 0x4 => {
                    if self.v[(msb & 0xF) as usize] != lsb {
                        self.pc += 2;
                        self.pc &= 0x0FFF;
                    }
                },
                (msb, lsb) if (msb >> 4) == 0x5 && (lsb & 0xF) == 0 => {
                    if self.v[(msb & 0xF) as usize] == self.v[(lsb >> 4) as usize] {
                        self.pc += 2;
                        self.pc &= 0x0FFF;
                    }
                },
                (msb, lsb) if (msb >> 4) == 0x6 => {
                    self.v[(msb & 0xF) as usize] = lsb;
                },
                (msb, lsb) if (msb >> 4) == 0x7 => {
                    self.v[(msb & 0xF) as usize] += lsb;
                },
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 0 => {
                    self.v[(msb & 0xF) as usize] = self.v[(lsb >> 4) as usize];
                },
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 1 => {
                    self.v[(msb & 0xF) as usize] |= self.v[(lsb >> 4) as usize]
                },
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 2 => {
                    self.v[(msb & 0xF) as usize] &= self.v[(lsb >> 4) as usize]
                },
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 3 => {
                    self.v[(msb & 0xF) as usize] ^= self.v[(lsb >> 4) as usize]
                },
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 4 => {
                    let (result, carry) = self.v[(msb & 0xF) as usize]
                        .overflowing_add(self.v[(lsb >> 4) as usize]);
                    self.v[(msb & 0xF) as usize] = result;
                    self.v[0xF] = carry as u8;
                },
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 5 => {
                    let (result, carry) = self.v[(msb & 0xF) as usize]
                        .overflowing_sub(self.v[(lsb >> 4) as usize]);
                    self.v[(msb & 0xF) as usize] = result;
                    self.v[0xF] = !carry as u8;
                },
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 6 => {
                    let x = msb & 0xF;

                    self.v[0xF] = self.v[x as usize] & 1;
                    self.v[x as usize] >>= 1;
                },
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 7 => {
                    let (result, carry) = self.v[(lsb >> 4) as usize]
                        .overflowing_sub(self.v[(msb & 0xF) as usize]);
                    self.v[(msb & 0xF) as usize] = result;
                    self.v[0xF] = !carry as u8;
                },
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 0xE => {
                    let x = msb & 0xF;

                    self.v[0xF] = (self.v[x as usize] & 0x80) >> 7;
                    self.v[x as usize] <<= 1;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn run_test(opcode: u16, setup: impl FnOnce(&mut Cpu)) -> Cpu {
        let mut cpu = Cpu::new();

        setup(&mut cpu);
        
        cpu.memory[cpu.pc as usize] = (opcode >> 8) as u8;
        cpu.memory[(cpu.pc+1) as usize] = (opcode & 0x00FF) as u8;

        cpu.cycle();
        cpu
    }

    #[test]
    fn test_advance_pc() { assert_eq!(run_test(0x00E0, |_| {}).pc, 0x202); }

    #[test]
    fn test_pc_overflow() { assert_eq!(run_test(0x00E0, |c| {c.pc = 0xFFE}).pc, 0x000); }

    #[test]
    fn test_cls() { assert_eq!(run_test(0x00E0, |c| {c.frame_buffer[0] = 1}).frame_buffer[0], 0); }

    #[test]
    fn test_ret() {
        let cpu = run_test(0x00EE, |c| {c.stack[0] = 0x3939; c.stack_pointer = 1});
        assert_eq!(cpu.stack_pointer, 0);
        assert_eq!(cpu.pc, 0x0939);
    }

    #[test]
    fn test_ret_underflow() {
        let cpu = run_test(0x00EE, |_| {});
        assert_eq!(cpu.stack_pointer, 15);
        assert_eq!(cpu.pc, 0x000);
    }

    #[test]
    fn test_jmp() { assert_eq!(run_test(0x1420, |_| {}).pc, 0x420); }

    #[test]
    fn test_cal() {
        let cpu = run_test(0x2420, |_| {});
        assert_eq!(cpu.stack[0], 0x202);
        assert_eq!(cpu.stack_pointer, 1);
        assert_eq!(cpu.pc, 0x420);
    }

    #[test]
    fn test_cal_overflow() {
        let cpu = run_test(0x2420, |c| {c.stack_pointer = 15});
        assert_eq!(cpu.stack[15], 0x202);
        assert_eq!(cpu.stack_pointer, 0);
        assert_eq!(cpu.pc, 0x420);
    }

    #[test]
    fn test_seb_eq() { assert_eq!(run_test(0x3969, |c| {c.v[9] = 0x69}).pc, 0x204); }

    #[test]
    fn test_seb_ne() { assert_eq!(run_test(0x3969, |c| {c.v[9] = 0x39}).pc, 0x202); }

    #[test]
    fn test_sne_eq() { assert_eq!(run_test(0x4269, |c| {c.v[2] = 0x69}).pc, 0x202); }

    #[test]
    fn test_sne_ne() { assert_eq!(run_test(0x4269, |c| {c.v[2] = 0x39}).pc, 0x204); }

    #[test]
    fn test_ser_eq() { assert_eq!(run_test(0x5420, |c| {c.v[4] = 0x39; c.v[2] = 0x39}).pc, 0x204); }

    #[test]
    fn test_ser_ne() { assert_eq!(run_test(0x5420, |c| {c.v[4] = 0x39; c.v[2] = 0x42}).pc, 0x202); }

    #[test]
    fn test_ldvb() { assert_eq!(run_test(0x6969, |_| {}).v[9], 0x69); }

    #[test]
    fn test_add() { assert_eq!(run_test(0x7777, |c| {c.v[7] = 0x01}).v[7], 0x78); }

    #[test]
    fn test_ldvv() { assert_eq!(run_test(0x8120, |c| {c.v[1] = 0x69; c.v[2] = 0x39}).v[1], 0x39); }

    #[test]
    fn test_or() { assert_eq!(run_test(0x8121, |c| {c.v[1] = 0x69; c.v[2] = 0x39}).v[1], 0x79); }

    #[test]
    fn test_and() { assert_eq!(run_test(0x8122, |c| {c.v[1] = 0x69; c.v[2] = 0x39}).v[1], 0x29); }

    #[test]
    fn test_xor() { assert_eq!(run_test(0x8123, |c| {c.v[1] = 0x69; c.v[2] = 0x39}).v[1], 0x50); }

    #[test]
    fn test_addc_coff() {
        let cpu = run_test(0x8124, |c| {c.v[1] = 0x69; c.v[2] = 0x39});
        assert_eq!(cpu.v[1], 0xA2);
        assert_eq!(cpu.v[15], 0x00);
    }

    #[test]
    fn test_addc_con() {
        let cpu = run_test(0x8124, |c| {c.v[1] = 0x69; c.v[2] = 0xFF});
        assert_eq!(cpu.v[1], 0x68);
        assert_eq!(cpu.v[15], 0x01);
    }

    #[test]
    fn test_subc_coff() {
        let cpu = run_test(0x8125, |c| {c.v[1] = 0x39; c.v[2] = 0x69});
        assert_eq!(cpu.v[1], 0xD0);
        assert_eq!(cpu.v[15], 0x00);
    }

    #[test]
    fn test_subc_con() {
        let cpu = run_test(0x8125, |c| {c.v[1] = 0x69; c.v[2] = 0x39});
        assert_eq!(cpu.v[1], 0x30);
        assert_eq!(cpu.v[15], 0x01);
    }

    #[test]
    fn test_shr() {
        let cpu = run_test(0x8106, |c| {c.v[1] = 0x69});
        assert_eq!(cpu.v[1], 0x34);
        assert_eq!(cpu.v[15], 0x1);
    }

    #[test]
    fn test_sbn_coff() {
        let cpu = run_test(0x8127, |c| {c.v[1] = 0x69; c.v[2] = 0x39});
        assert_eq!(cpu.v[1], 0xD0);
        assert_eq!(cpu.v[15], 0x00);
    }

    #[test]
    fn test_sbn_con() {
        let cpu = run_test(0x8127, |c| {c.v[1] = 0x39; c.v[2] = 0x69});
        assert_eq!(cpu.v[1], 0x30);
        assert_eq!(cpu.v[15], 0x01);
    }

    #[test]
    fn test_shl() {
        let cpu = run_test(0x810E, |c| {c.v[1] = 0x69});
        assert_eq!(cpu.v[1], 0xD2);
        assert_eq!(cpu.v[15], 0x0);
    }
}
