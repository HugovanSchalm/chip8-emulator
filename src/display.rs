use minifb::{Key, Window, WindowOptions, Scale};

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;

const ON_COLOR: u32 = 0x00FF00; // Green
const OFF_COLOR: u32 = 0x0; // Black

pub struct Display {
    window: Window,
}

impl Display {
    pub fn new() -> Display {
        let window = Window::new(
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

        let frame_buffer = vec![vec![false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];

        let mut display = Display { window };

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
}
