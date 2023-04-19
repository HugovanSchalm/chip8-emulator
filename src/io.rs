use minifb::{Key, Window, WindowOptions, Scale};

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;

const ON_COLOR: u32 = 0x00FF00; // Green
const OFF_COLOR: u32 = 0x0; // Black

pub struct IO {
    window: Window,
}

impl IO {
    pub fn new() -> IO {
        let mut window = Window::new(
            "Chip-8 emulator",
            DISPLAY_WIDTH,
            DISPLAY_HEIGHT,
            WindowOptions {
                borderless: false,
                title: true,
                resize: true,
                scale: Scale::X16,
                scale_mode: minifb::ScaleMode::AspectRatioStretch,
                topmost: false,
                transparency: false,
                none: false,
            },
        )
        .unwrap();

        window.limit_update_rate(None);

        let frame_buffer = vec![vec![false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];

        let mut display = IO { window };

        display.refresh(&frame_buffer);

        display
    }

    pub fn refresh(&mut self, frame_buffer: &Vec<Vec<bool>>) {
        let mut buffer: Vec<u32> = Vec::with_capacity(DISPLAY_WIDTH * DISPLAY_HEIGHT);
        for y in 0..frame_buffer.len() {
            for x in 0..frame_buffer[y].len() {
                if frame_buffer[y][x] {
                    buffer.push(ON_COLOR);
                } else {
                    buffer.push(OFF_COLOR);
                }
            }
        }

        self.window
            .update_with_buffer(&buffer, DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .unwrap();
    }

    pub fn should_stay_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }

    pub fn get_keys(&self) -> [bool; 16] {
        let mut keys = [false; 16];

        self.window.get_keys().iter().for_each(|key|
            match key {
                Key::Key1 => keys[1] = true,
                Key::Key2 => keys[2] = true,
                Key::Key3 => keys[3] = true,
                Key::Key4 => keys[0xC] = true,
                Key::Q => keys[4] = true,
                Key::W => keys[5] = true,
                Key::E => keys[6] = true,
                Key::R => keys[0xD] = true,
                Key::A => keys[7] = true,
                Key::S => keys[8] = true,
                Key::D => keys[9] = true,
                Key::F => keys[0xE] = true,
                Key::Z => keys[0xA] = true,
                Key::X => keys[0] = true,
                Key::C => keys[0xB] = true,
                Key::V => keys[0xF] = true,
                _ => {}
            }
        );

        keys
    }
}
