use crate::io::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::font::FONT;
use rand::prelude::*;
use std::time::{Duration, Instant};

pub struct Processor {
    ram: Vec<u8>,
    framebuffer: Vec<Vec<bool>>,
    pc: usize,
    i: usize,
    stack: Vec<usize>,
    delay_timer: u8,
    sound_timer: u8,
    registers: Vec<u8>,
    rng: ThreadRng,
    keys: [bool; 16],
    last_timer_update: Instant
}

impl Processor {
    pub fn new() -> Processor {
        let mut ram = vec![0u8; 4096];

        for i in 0..FONT.len() {
            ram[i] = FONT[i];
        }

        Processor {
            ram,
            framebuffer: vec![vec![false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            pc: 0x200,
            i: 0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            registers: vec![0u8; 16],
            rng: thread_rng(),
            keys: [false; 16],
            last_timer_update: Instant::now()
        }
    }

    pub fn load_data(&mut self, data: &[u8]) {
        for (i, val) in data.iter().enumerate() {
            self.ram[0x200 + i] = *val;
        }
    }

    pub fn set_keys(&mut self, keys: &[bool; 16]) {
        self.keys = *keys;
    }

    pub fn step(&mut self) -> bool {
        let mut update_timers = false;
        if self.last_timer_update.elapsed() > Duration::from_secs_f32(1f32/60f32) {
            update_timers = true;
            self.last_timer_update = Instant::now();
        }

        if self.delay_timer > 0 && update_timers {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 && update_timers {
            self.sound_timer -= 1;
        }

        let mut vram_changed = false;

        // Fetch instruction
        let instruction = (self.ram[self.pc] as u16) << 8 | self.ram[self.pc + 1] as u16;
        // println!("{:#04x}", instruction);
        self.pc += 2;

        // Turn into nibbles for identifying commands
        let nibbles = (
            ((instruction & 0xF000) >> 12) as u8,
            ((instruction & 0x0F00) >> 8) as u8,
            ((instruction & 0x00F0) >> 4) as u8,
            (instruction & 0x000F) as u8,
        );

        match nibbles {
            (0x0, 0x0, 0xe, 0x0) => {
                for i in 0..self.framebuffer.len() {
                    for j in 0..self.framebuffer[i].len() {
                        self.framebuffer[i][j] = false;
                    }
                }
                vram_changed = true;
            }
            (0x0, 0x0, 0xE, 0xE) => {
                self.pc = self.stack.pop().unwrap();
            }
            (0x1, ..) => {
                self.pc = (instruction & 0x0FFF) as usize;
            }
            (0x2, ..) => {
                self.stack.push(self.pc);
                self.pc = (instruction & 0x0FFF) as usize;
            }
            (0x3, ..) => {
                if self.registers[nibbles.1 as usize] == (instruction & 0xFF) as u8 {
                    self.pc += 2;
                }
            }
            (0x4, ..) => {
                if self.registers[nibbles.1 as usize] != (instruction & 0xFF) as u8 {
                    self.pc += 2;
                }
            }
            (0x5, _, _, 0) => {
                if self.registers[nibbles.1 as usize] == self.registers[nibbles.2 as usize] {
                    self.pc += 2;
                }
            }
            (0x6, ..) => {
                self.registers[nibbles.1 as usize] = (instruction & 0x00FF) as u8;
            }
            (0x7, ..) => {
                self.registers[nibbles.1 as usize] =
                    self.registers[nibbles.1 as usize].wrapping_add((instruction & 0x00FF) as u8);
            }
            (0x8, _, _, 0) => {
                self.registers[nibbles.1 as usize] = self.registers[nibbles.2 as usize];
            }
            (0x8, _, _, 1) => {
                self.registers[nibbles.1 as usize] |= self.registers[nibbles.2 as usize];
            }
            (0x8, _, _, 2) => {
                self.registers[nibbles.1 as usize] &= self.registers[nibbles.2 as usize];
            }
            (0x8, _, _, 3) => {
                self.registers[nibbles.1 as usize] ^= self.registers[nibbles.2 as usize];
            }
            (0x8, _, _, 4) => {
                let overflow;

                (self.registers[nibbles.1 as usize], overflow) = self.registers[nibbles.1 as usize]
                    .overflowing_add(self.registers[nibbles.2 as usize]);

                if overflow {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
            }
            (0x8, _, _, 5) => {
                let x = self.registers[nibbles.1 as usize];
                let y = self.registers[nibbles.2 as usize];

                let overflow;

                if y > x {
                    overflow = true;
                } else {
                    overflow = false;
                }

                self.registers[nibbles.1 as usize] = x.wrapping_sub(y);

                if overflow {
                    self.registers[0xF] = 0;
                } else {
                    self.registers[0xF] = 1;
                }
            }
            (0x8, _, _, 0x7) => {
                let x = self.registers[nibbles.1 as usize];
                let y = self.registers[nibbles.2 as usize];

                let overflow;

                if x > y {
                    overflow = true;
                } else {
                    overflow = false;
                }

                self.registers[nibbles.1 as usize] = y.wrapping_sub(x);

                if overflow {
                    self.registers[0xF] = 0;
                } else {
                    self.registers[0xF] = 1;
                }
            }
            (0x8, _, _, 0x6) => {
                let y = self.registers[nibbles.2 as usize];
                self.registers[nibbles.1 as usize] = y >> 1;
                self.registers[0xF] = y & 1;
            }
            (0x8, _, _, 0xE) => {
                let y = self.registers[nibbles.2 as usize];
                self.registers[nibbles.1 as usize] = y << 1;
                self.registers[0xF] = (y >> 7) & 1;
            }
            (0x9, _, _, 0) => {
                if self.registers[nibbles.1 as usize] != self.registers[nibbles.2 as usize] {
                    self.pc += 2;
                }
            }
            (0xA, ..) => {
                self.i = (instruction & 0x0FFF) as usize;
            }
            (0xB, ..) => {
                self.pc = (instruction & 0x0FFF) as usize + self.registers[0] as usize;
            }
            (0xC, ..) => {
                self.registers[nibbles.1 as usize] = (instruction & 0xFF) as u8 & self.rng.gen::<u8>();
            }
            (0xD, ..) => {
                self.registers[0xF] = 0;

                let x_reg = nibbles.1 as usize;
                let y_reg = nibbles.2 as usize;

                for byte in 0..nibbles.3 as usize {
                    let y = (self.registers[y_reg] as usize + byte) % DISPLAY_HEIGHT;
                    for bit in 0..8 {
                        let x = (self.registers[x_reg] as usize + bit) % DISPLAY_WIDTH;
                        if x >= self.framebuffer[y].len() {
                            break;
                        }
                        let val = (self.ram[self.i + byte] >> (7 - bit)) & 1 == 1;
                        if self.framebuffer[y][x] && val {
                            self.registers[0xF] = 1;
                        }
                        self.framebuffer[y][x] ^= val;
                    }
                }
                vram_changed = true;
            }
            (0xE, _, 0x9, 0xE) => {
                let key = self.registers[nibbles.1 as usize] as usize;
                if self.keys[key] {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 0x1) => {
                let key = self.registers[nibbles.1 as usize] as usize;
                if !self.keys[key] {
                    self.pc += 2;
                }
            }
            (0xF, _, 0x0, 0x7) => {
                self.registers[nibbles.1 as usize] = self.delay_timer;
            }
            (0xF, _, 0x0, 0xA) => {
                let mut pressed = false;
                for i in 0..16 {
                    if self.keys[i] {
                        self.registers[nibbles.1 as usize] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc -= 2;
                }
            }
            (0xF, _, 0x1, 0x5) => {
                self.delay_timer = self.registers[nibbles.1 as usize];
            }
            (0xF, _, 0x1, 0x8) => {
                self.sound_timer = self.registers[nibbles.1 as usize];
            }
            (0xF, _, 0x1, 0xE) => {
                self.i += self.registers[nibbles.1 as usize] as usize;
                if self.i >= 0x1000 {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
            }
            (0xF, _, 0x2, 0x9) => {
                self.i = self.registers[nibbles.1 as usize] as usize * 5;
            }
            (0xF, _, 0x3, 0x3) => {
                let x = self.registers[nibbles.1 as usize];
                let d1 = x / 100;
                let d2 = x % 100 / 10;
                let d3 = x % 10;
                self.ram[self.i] = d1;
                self.ram[self.i + 1] = d2;
                self.ram[self.i + 2] = d3;
            }
            (0xF, _, 0x5, 0x5) => {
                for offset in 0..=nibbles.1 as usize {
                    self.ram[self.i + offset] = self.registers[offset];
                }
            }
            (0xF, _, 0x6, 0x5) => {
                for offset in 0..=nibbles.1 as usize {
                    self.registers[offset] = self.ram[self.i + offset];
                }
            }
            _ => {}
        }

        return vram_changed;
    }

    pub fn get_framebuffer(&self) -> &Vec<Vec<bool>> {
        &self.framebuffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loading_data() {
        let mut processor = Processor::new();
        let data: [u8; 3] = [0x1, 0x2, 0x3];
        processor.load_data(&data);

        assert_eq!(processor.ram[0x200], 0x1);
        assert_eq!(processor.ram[0x201], 0x2);
        assert_eq!(processor.ram[0x202], 0x3);
    }

    #[test]
    fn clear_screen() {
        let mut processor: Processor = Processor::new();
        processor.framebuffer[5][13] = true;
        processor.framebuffer[8][4] = true;
        processor.framebuffer[3][9] = true;
        processor.load_data(&[0x0, 0xE0]);
        assert!(processor.step());

        for i in 0..processor.framebuffer.len() {
            for j in 0..processor.framebuffer.len() {
                assert!(!processor.framebuffer[i][j]);
            }
        }
    }

    #[test]
    fn jump() {
        let mut processor = Processor::new();
        processor.load_data(&[0x14, 0x11]);
        assert!(!processor.step());
        assert_eq!(processor.pc, 0x0411);
    }

    #[test]
    fn set_register_x() {
        let mut processor = Processor::new();
        processor.load_data(&[0x64, 0x3F]);
        assert!(!processor.step());
        assert_eq!(processor.registers[0x4], 0x3F);
    }

    #[test]
    fn add_value_to_register_x() {
        let mut processor = Processor::new();
        processor.load_data(&[0x68, 0x05, 0x78, 0xF1]);
        assert!(!processor.step());
        assert!(!processor.step());
        assert_eq!(processor.registers[0x8], 0xF6);
    }

    #[test]
    fn set_index_register() {
        let mut processor = Processor::new();
        processor.load_data(&[0xA1, 0x23]);
        assert!(!processor.step());
        assert_eq!(processor.i, 0x123);
    }

    #[test]
    fn test_draw() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0x01, 0x61, 0x01, 0xA2, 0x08, 0xD0, 0x11, 0b10101010]);
        processor.step();
        processor.step();
        processor.step();
        processor.step();
        assert!(processor.framebuffer[1][1]);
        assert!(!processor.framebuffer[1][2]);
        assert!(processor.framebuffer[1][3]);
        assert!(!processor.framebuffer[1][4]);
        assert!(processor.framebuffer[1][5]);
        assert!(!processor.framebuffer[1][6]);
        assert!(processor.framebuffer[1][7]);
        assert!(!processor.framebuffer[1][8]);
    }

    #[test]
    fn test_skip_x_equal() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0x22, 0x30, 0x05, 0x30, 0x22]);
        processor.step();
        processor.step();
        assert_eq!(processor.pc, 0x204);
        processor.step();
        assert_eq!(processor.pc, 0x208);
    }

    #[test]
    fn test_skip_x_not_equal() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0x22, 0x40, 0x22, 0x40, 0x05]);
        processor.step();
        processor.step();
        assert_eq!(processor.pc, 0x204);
        processor.step();
        assert_eq!(processor.pc, 0x208);
    }

    #[test]
    fn test_skip_x_y_equal() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0x12, 0x61, 0x08, 0x50, 0x10, 0x61, 0x12, 0x50, 0x10]);
        processor.step();
        processor.step();
        processor.step();
        assert_eq!(processor.pc, 0x206);
        processor.step();
        processor.step();
        assert_eq!(processor.pc, 0x20C);
    }

    #[test]
    fn test_skip_x_y_not_equal() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0x12, 0x61, 0x12, 0x90, 0x10, 0x61, 0xFF, 0x90, 0x10]);
        processor.step();
        processor.step();
        processor.step();
        assert_eq!(processor.pc, 0x206);
        processor.step();
        processor.step();
        assert_eq!(processor.pc, 0x20C);
    }

    #[test]
    fn test_set_x_y() {
        let mut processor = Processor::new();
        processor.load_data(&[0x64, 0x12, 0x87, 0x40]);
        processor.step();
        processor.step();
        assert_eq!(processor.registers[0x7], 0x12);
    }

    #[test]
    fn test_or() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0b11001010, 0x61, 0b00101010, 0x80, 0x11]);
        processor.step();
        processor.step();
        processor.step();
        assert_eq!(processor.registers[0], 0b11001010 | 0b00101010);
        assert_eq!(processor.registers[1], 0b00101010);
    }

    #[test]
    fn test_and() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0b11001010, 0x61, 0b00101010, 0x80, 0x12]);
        processor.step();
        processor.step();
        processor.step();
        assert_eq!(processor.registers[0], 0b11001010 & 0b00101010);
        assert_eq!(processor.registers[1], 0b00101010);
    }

    #[test]
    fn test_xor() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0b11001010, 0x61, 0b00101010, 0x80, 0x13]);
        processor.step();
        processor.step();
        processor.step();
        assert_eq!(processor.registers[0], 0b11001010 ^ 0b00101010);
        assert_eq!(processor.registers[1], 0b00101010);
    }

    #[test]
    fn test_add_flag() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0xFE, 0x61, 0x01, 0x80, 0x14, 0x80, 0x14]);
        processor.step();
        processor.step();
        processor.step();
        assert_eq!(processor.registers[0], 0xFF);
        assert_eq!(processor.registers[0xF], 0);
        processor.step();
        assert_eq!(processor.registers[0], 0);
        assert_eq!(processor.registers[0xF], 1);
    }

    #[test]
    fn test_subtract_y_from_x() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0xF, 0x61, 0xA, 0x80, 0x15, 0x80, 0x15]);
        processor.step();
        processor.step();
        processor.step();
        assert_eq!(processor.registers[0], 0x05);
        assert_eq!(processor.registers[0xF], 1);
        processor.step();
        assert_eq!(processor.registers[0], 251);
        assert_eq!(processor.registers[0xF], 0);
    }

    #[test]
    fn test_subtract_x_from_y() {
        let mut processor = Processor::new();
        processor.load_data(&[
            0x60, 0xA, 0x61, 0xF, 0x80, 0x17, 0x60, 0x01, 0x61, 0x00, 0x80, 0x17,
        ]);
        processor.step();
        processor.step();
        processor.step();
        assert_eq!(processor.registers[0], 0x05);
        assert_eq!(processor.registers[0xF], 1);
        processor.step();
        processor.step();
        processor.step();
        assert_eq!(processor.registers[0], 0xFF);
        assert_eq!(processor.registers[0xF], 0);
    }

    #[test]
    fn test_shift_right() {
        let mut processor = Processor::new();
        processor.load_data(&[0x61, 0b01010101, 0x80, 0x16, 0x61, 0b10000000, 0x80, 0x16]);
        processor.step();
        processor.step();
        assert_eq!(processor.registers[0], 0b00101010);
        assert_eq!(processor.registers[0xF], 1);
        processor.step();
        processor.step();
        assert_eq!(processor.registers[0], 0b01000000);
        assert_eq!(processor.registers[0xF], 0);
    }

    #[test]
    fn test_shift_left() {
        let mut processor = Processor::new();
        processor.load_data(&[0x61, 0b01010101, 0x80, 0x1E, 0x61, 0b10000000, 0x80, 0x1E]);
        processor.step();
        processor.step();
        assert_eq!(processor.registers[0], 0b01010101 << 1);
        assert_eq!(processor.registers[0xF], 0);
        processor.step();
        processor.step();
        assert_eq!(processor.registers[0], 0);
        assert_eq!(processor.registers[0xF], 1);
    }

    #[test]
    fn test_jump_with_offset() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0x05, 0xB0, 0x20]);
        processor.step();
        processor.step();
        assert_eq!(processor.pc, 0x25);
    }

    #[test]
    fn test_subroutine() {
        let mut processor = Processor::new();
        processor.load_data(&[0x21, 0x23]);
        processor.step();
        assert_eq!(processor.stack.len(), 1);
        assert_eq!(processor.pc, 0x123);
        assert_eq!(processor.stack[0], 0x202);
    }

    #[test]
    fn test_return_subroutine() {
        let mut processor = Processor::new();
        processor.ram[0x124] = 0x00;
        processor.ram[0x125] = 0xEE;
        processor.load_data(&[0x21, 0x24]);
        processor.step();
        processor.step();
        assert_eq!(processor.pc, 0x202);
        assert_eq!(processor.stack.len(), 0);
    }

    #[test]
    fn test_set_delay_timer() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0x12, 0xF0, 0x15, 0x60, 0x45]);
        processor.step();
        processor.step();
        assert_eq!(processor.delay_timer, 0x12);
        processor.step();
        assert_eq!(processor.delay_timer, 0x11);
    }

    #[test]
    fn test_get_delay_timer() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0x12, 0xF0, 0x15, 0xF1, 0x07]);
        processor.step();
        processor.step();
        assert_eq!(processor.delay_timer, 0x12);
        processor.step();
        assert_eq!(processor.delay_timer, 0x11);
        assert_eq!(processor.registers[1], 0x11);
    }

    #[test]
    fn test_set_sound_timer() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0x12, 0xF0, 0x18, 0x60, 0x45]);
        processor.step();
        processor.step();
        assert_eq!(processor.sound_timer, 0x12);
        processor.step();
        assert_eq!(processor.sound_timer, 0x11);
    }

    #[test]
    fn test_add_to_index() {
        let mut processor = Processor::new();
        processor.load_data(&[0xA1, 0x23, 0x60, 0x01, 0xF0, 0x1E]);
        processor.step();
        processor.step();
        processor.step();
        assert_eq!(processor.i, 0x124);
    }

    #[test]
    fn test_font_character() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0x0D, 0xF0, 0x29]);
        processor.step();
        processor.step();
        assert_eq!(processor.i, 0xD);
    }

    #[test]
    fn test_decimal_conversion() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0x9C, 0xA5, 0x67, 0xF0, 0x33]);
        processor.step();
        processor.step();
        processor.step();
        assert_eq!(processor.ram[0x567], 1);
        assert_eq!(processor.ram[0x568], 5);
        assert_eq!(processor.ram[0x569], 6);
    }

    #[test]
    fn test_store() {
        let mut processor = Processor::new();
        processor.load_data(&[0x60, 0x01, 0x61, 0x02, 0x62, 0x03, 0xA5, 0x00, 0xF2, 0x55]);
        for _ in 0..5 {
            processor.step();
        }
        assert_eq!(processor.ram[0x500], 0x01);
        assert_eq!(processor.ram[0x501], 0x02);
        assert_eq!(processor.ram[0x502], 0x03);
    }

    #[test]
    fn test_load() {
        let mut processor = Processor::new();
        processor.ram[0x500] = 0x10;
        processor.ram[0x501] = 0x20;
        processor.ram[0x502] = 0x30;
        processor.load_data(&[0x60, 0x01, 0x61, 0x02, 0x62, 0x03, 0xA5, 0x00, 0xF2, 0x65]);
        for _ in 0..5 {
            processor.step();
        }
        assert_eq!(processor.registers[0], 0x10);
        assert_eq!(processor.registers[1], 0x20);
        assert_eq!(processor.registers[2], 0x30);
    }
}
