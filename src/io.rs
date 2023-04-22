use minifb::{Key, Menu, Scale, Window, WindowOptions};
use std::time::Duration;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;

pub const MENU_OPEN_FILE_ID: usize = 0;
pub const MENU_RESET_ID: usize = 1;

pub const MENU_MODE_CHIP8_ID: usize = 2;
pub const MENU_MODE_SUPERCHIP_ID: usize = 3;
pub const MENU_MODE_XOCHIP_ID: usize = 4;
pub const MENU_COLOR_MATRIX_ID: usize = 5;
pub const MENU_COLOR_NEON_ID: usize = 6;
pub const MENU_COLOR_OLDSCHOOL_ID: usize = 7;

pub struct IO {
    window: Window,
    framebuffer: Vec<Vec<bool>>,
    on_color: u32,
    off_color: u32,
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

        let mut file_menu = Menu::new("File").unwrap();
        file_menu.add_item("Open", MENU_OPEN_FILE_ID).build();
        file_menu.add_item("Reset", MENU_RESET_ID).build();

        let mut options_menu = Menu::new("Options").unwrap();

        let mut mode_menu = Menu::new("Mode").unwrap();
        mode_menu.add_item("Chip-8", MENU_MODE_CHIP8_ID).build();
        mode_menu
            .add_item("SUPER-CHIP", MENU_MODE_SUPERCHIP_ID)
            .build();
        mode_menu.add_item("XO-CHIP", MENU_MODE_XOCHIP_ID).build();

        let mut color_menu = Menu::new("Colors").unwrap();
        color_menu.add_item("Matrix", MENU_COLOR_MATRIX_ID).build();
        color_menu.add_item("Neon", MENU_COLOR_NEON_ID).build();
        color_menu.add_item("Old School", MENU_COLOR_OLDSCHOOL_ID).build();

        options_menu.add_sub_menu("Mode", &mode_menu);
        options_menu.add_sub_menu("Colors", &color_menu);

        window.add_menu(&file_menu);
        window.add_menu(&options_menu);

        window.limit_update_rate(Some(Duration::from_secs_f64(1f64 / 60f64))); //

        let framebuffer = vec![vec![false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];

        let on_color: u32 = 0x00FF00; // Green
        let off_color: u32 = 0x0; // Black

        let mut display = IO {
            window,
            framebuffer,
            on_color,
            off_color,
        };

        display.refresh_display();

        display
    }

    pub fn set_framebuffer(&mut self, other: &Vec<Vec<bool>>) {
        for x in 0..other.len() {
            for y in 0..other[x].len() {
                self.framebuffer[x][y] = other[x][y]
            }
        }
    }

    fn handle_menus(&mut self) {
        if let Some(menu_id) = self.window.is_menu_pressed() {
            match menu_id {
                MENU_COLOR_MATRIX_ID => {
                    self.on_color = 0x00FF00; // Green
                    self.off_color = 0x0; // Black
                }
                MENU_COLOR_NEON_ID => {
                    self.on_color = 0x00FFEC;
                    self.off_color = 0xD600FF;
                }
                MENU_COLOR_OLDSCHOOL_ID => {
                    self.on_color = 0xF0F6F0;
                    self.off_color = 0x222323;
                }
                _ => {}
            }
        }
    }

    pub fn refresh_display(&mut self) {
        self.handle_menus();

        let mut buffer_for_screen = Vec::with_capacity(DISPLAY_WIDTH * DISPLAY_HEIGHT);
        for y in 0..self.framebuffer.len() {
            for x in 0..self.framebuffer[y].len() {
                if self.framebuffer[y][x] {
                    buffer_for_screen.push(self.on_color);
                } else {
                    buffer_for_screen.push(self.off_color);
                }
            }
        }
        self.window
            .update_with_buffer(&buffer_for_screen, DISPLAY_WIDTH, DISPLAY_HEIGHT)
            .unwrap();
    }

    pub fn should_stay_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }

    pub fn get_keys(&self) -> [bool; 16] {
        let mut keys = [false; 16];

        self.window.get_keys().iter().for_each(|key| match key {
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
        });

        keys
    }
}
