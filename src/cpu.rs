use rand::random;

pub struct Cpu {
    pub memory: [u8; 4096],
    pub v: [u8; 16],
    pub index: u16,
    pub stack: [u16; 32],
    pub stack_pointer: u8,
    pub sound_timer: u8,
    pub delay_timer: u8,
    pub frame_buffer: [u64; 32],
    pub pc: u16,
}

impl Cpu {
    pub fn new() -> Self {
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

    pub fn load_memory(&mut self, rom: &[u8]) {
        for (i, byte) in FONT_DATA.iter().enumerate() {
            self.memory[i] = *byte;
        }

        for (i, byte) in rom.iter().enumerate() {
            self.memory[0x200 + i] = *byte;
        }
    }

    pub fn cycle(&mut self) {
        if let (Some(msb), Some(lsb)) = (
            self.memory.get(self.pc as usize),
            self.memory.get((self.pc + 1) as usize),
        ) {
            self.pc += 2;
            self.pc &= 0x0FFF;

            match (*msb, *lsb) {
                (0x00, 0xE0) => {
                    self.frame_buffer = [0; 32];
                }
                (0x00, 0xEE) => {
                    if self.stack_pointer == 0 {
                        eprintln!("WARNING: stack underflow");
                    }

                    self.stack_pointer = self.stack_pointer.wrapping_sub(1) & 15;
                    self.pc = self.stack[self.stack_pointer as usize] & 0x0FFF;
                }
                (msb, lsb) if ((msb >> 4) & 0xF) == 0x1 => {
                    self.pc = u16::from_be_bytes([msb & 0xF, lsb]);
                }
                (msb, lsb) if ((msb >> 4) & 0xF) == 0x2 => {
                    if self.stack_pointer == 16 {
                        eprintln!("WARNING: stack overflow");
                    }

                    self.stack[self.stack_pointer as usize] = self.pc;
                    self.pc = u16::from_be_bytes([msb & 0xF, lsb]);
                    self.stack_pointer = self.stack_pointer.wrapping_add(1) & 15;
                }
                (msb, lsb) if (msb >> 4) == 0x3 => {
                    if self.v[(msb & 0xF) as usize] == lsb {
                        self.pc += 2;
                        self.pc &= 0x0FFF;
                    }
                }
                (msb, lsb) if (msb >> 4) == 0x4 => {
                    if self.v[(msb & 0xF) as usize] != lsb {
                        self.pc += 2;
                        self.pc &= 0x0FFF;
                    }
                }
                (msb, lsb) if (msb >> 4) == 0x5 && (lsb & 0xF) == 0 => {
                    if self.v[(msb & 0xF) as usize] == self.v[(lsb >> 4) as usize] {
                        self.pc += 2;
                        self.pc &= 0x0FFF;
                    }
                }
                (msb, lsb) if (msb >> 4) == 0x6 => {
                    self.v[(msb & 0xF) as usize] = lsb;
                }
                (msb, lsb) if (msb >> 4) == 0x7 => {
                    self.v[(msb & 0xF) as usize] = self.v[(msb & 0xF) as usize].wrapping_add(lsb);
                }
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 0 => {
                    self.v[(msb & 0xF) as usize] = self.v[(lsb >> 4) as usize];
                }
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 1 => {
                    self.v[(msb & 0xF) as usize] |= self.v[(lsb >> 4) as usize]
                }
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 2 => {
                    self.v[(msb & 0xF) as usize] &= self.v[(lsb >> 4) as usize]
                }
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 3 => {
                    self.v[(msb & 0xF) as usize] ^= self.v[(lsb >> 4) as usize]
                }
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 4 => {
                    let (result, carry) =
                        self.v[(msb & 0xF) as usize].overflowing_add(self.v[(lsb >> 4) as usize]);
                    self.v[(msb & 0xF) as usize] = result;
                    self.v[0xF] = carry as u8;
                }
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 5 => {
                    let (result, carry) =
                        self.v[(msb & 0xF) as usize].overflowing_sub(self.v[(lsb >> 4) as usize]);
                    self.v[(msb & 0xF) as usize] = result;
                    self.v[0xF] = !carry as u8;
                }
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 6 => {
                    let x = msb & 0xF;

                    self.v[0xF] = self.v[x as usize] & 1;
                    self.v[x as usize] >>= 1;
                }
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 7 => {
                    let (result, carry) =
                        self.v[(lsb >> 4) as usize].overflowing_sub(self.v[(msb & 0xF) as usize]);
                    self.v[(msb & 0xF) as usize] = result;
                    self.v[0xF] = !carry as u8;
                }
                (msb, lsb) if (msb >> 4) == 0x8 && (lsb & 0xF) == 0xE => {
                    let x = msb & 0xF;

                    self.v[0xF] = (self.v[x as usize] & 0x80) >> 7;
                    self.v[x as usize] <<= 1;
                }
                (msb, lsb) if (msb >> 4) == 0x9 && (lsb & 0xF) == 0x0 => {
                    if self.v[(msb & 0xF) as usize] != self.v[(lsb >> 4) as usize] {
                        self.pc += 2;
                        self.pc &= 0x0FFF;
                    }
                }
                (msb, lsb) if (msb >> 4) == 0xA => {
                    self.index = u16::from_be_bytes([msb & 0xF, lsb]);
                }
                (msb, lsb) if (msb >> 4) == 0xB => {
                    self.pc = u16::from_be_bytes([msb & 0xF, lsb]) + self.v[0] as u16;
                    self.pc &= 0x0FFF;
                }
                (msb, lsb) if (msb >> 4) == 0xC => {
                    let rand: u8 = random();
                    self.v[(msb & 0xF) as usize] = rand & lsb;
                }
                (msb, lsb) if (msb >> 4) == 0xD => {
                    let x_start = self.v[(msb & 0xF) as usize];
                    let y_start = self.v[(lsb >> 4) as usize];
                    let byte_num = lsb & 0xF;

                    let mut collision = false;

                    for offset in 0..byte_num {
                        let total_offset = (self.index + offset as u16) & 0xFFF;
                        let byte = self.memory[total_offset as usize];

                        for idx in 0..8 {
                            let bit = (byte >> 7 - idx) & 1;
                            let x = (x_start + idx) & 0x3F;
                            let y = (y_start + offset) & 0x1F;

                            if bit == 1 {
                                if ((self.frame_buffer[y as usize] >> (63 - x)) & 1) == 1 {
                                    collision = true;
                                }

                                let new_byte = 1 << (63 - x);
                                self.frame_buffer[y as usize] ^= new_byte;
                            }
                        }

                        self.v[0xF] = collision as u8;
                    }
                }
                _ => todo!(),
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
