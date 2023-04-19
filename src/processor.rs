use crate::display::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::font::FONT;

pub struct Processor {
    ram: Vec<u8>,
    framebuffer: Vec<Vec<bool>>,
    pc: usize,
    i: usize,
    stack: Vec<u32>,
    delay_timer: u8,
    sound_timer: u8,
    registers: Vec<u8>,
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
        }
    }

    pub fn load_data(&mut self, data: &[u8]) {
        for (i, val) in data.iter().enumerate() {
            self.ram[0x200 + i] = *val;
        }
    }

    pub fn step(&mut self) -> bool {
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
            (instruction & 0x000F) as u8
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
            (0x1, ..) => {
                self.pc = (instruction & 0x0FFF) as usize;
            }
            (0x6, ..) => {
                self.registers[nibbles.1 as usize] = (instruction & 0x00FF) as u8;
            }
            (0x7, ..) => {
                self.registers[nibbles.1 as usize] += (instruction & 0x00FF) as u8;
            }
            (0xA, ..) => {
                self.i = (instruction & 0x0FFF) as usize;
            }
            (0xD, ..) => {
                self.registers[0xF] = 0;

                let x_reg = nibbles.1 as usize;
                let y_reg = nibbles.2 as usize;

                for byte in 0..nibbles.3 as usize {
                    let y  = (self.registers[y_reg] as usize + byte) % DISPLAY_HEIGHT;
                    for bit in 0..8 {
                        let x = (self.registers[x_reg] as usize + bit) % DISPLAY_WIDTH;
                        if x >= self.framebuffer[y].len() {
                            break;
                        }
                        let val = (self.ram[self.i + byte] >> (7-bit)) & 1 == 1;
                        if self.framebuffer[y][x] && val {
                            self.registers[0xF] = 1;
                        }
                        self.framebuffer[y][x] ^= val;
                    }
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
        let data: [u8;3] = [0x1, 0x2, 0x3];
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
        assert_eq!(processor.pc, 0x212);
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
        assert_eq!(processor.pc, 0x212);
    }

    #[test]
    fn test_set_x_y() {
        let mut processor = Processor::new();
        processor.load_data(&[])
    }
}