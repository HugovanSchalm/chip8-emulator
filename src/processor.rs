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
            ram[0x050 + i] = FONT[i];
        }

        Processor {
            ram,
            framebuffer: vec![vec![false; DISPLAY_HEIGHT]; DISPLAY_WIDTH],
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
        let mut instruction = self.ram[self.pc] as usize;
        instruction = instruction << 8;
        instruction += self.ram[self.pc + 1] as usize;
        self.pc += 2;

        // Turn into nibbles for identifying commands
        let mut nibbles = vec![0usize; 4];
        nibbles[0] = (instruction & 0xF000) >> 24;
        nibbles[1] = (instruction & 0x0F00) >> 16;
        nibbles[2] = (instruction & 0x00F0) >> 8;
        nibbles[3] = instruction & 0x000F;

        match nibbles[0] {
            0x0 if nibbles[0] == 0x0 && nibbles[1] == 0xE && nibbles[2] == 0x0 => {
                for mut p in &self.framebuffer {
                    p = &false;
                }
                vram_changed = true;
            }
            0x1 => {
                self.pc = (instruction & 0x0FFF);
            }
            0x6 => {
                self.registers[nibbles[1]] = (instruction & 0x00FF) as u8;
            }
            0x7 => {
                self.registers[nibbles[1]] += (instruction & 0x00FF) as u8;
            }
            0xA => {
                self.i = instruction & 0x0FFF;
            }
            0xD => {
                let x = nibbles[1] % DISPLAY_WIDTH;
                let y = nibbles[2] % DISPLAY_HEIGHT;
                self.registers[0xF] = 0;
                for i in 0..nibbles[3] {
                    let sprite_data = self.ram[self.i];
                    for (j, mut pixel) in self.framebuffer[y][x..x+8].into_iter().enumerate() {
                        pixel = sprite_data.as_bi[j] 
                    }
                }
            }
            _ => {}
        }

        return vram_changed;
    }
}
