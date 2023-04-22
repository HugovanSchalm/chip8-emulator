#![windows_subsystem = "windows"]

use chip8_emulator::{io, rom, processor::{self, Processor}, splash};
use native_dialog::FileDialog;

const INSTRUCTIONS_PER_FRAME: u32 = 10;

fn main() {
    let mut processor = processor::Processor::new();

    processor.load_data(&splash::SPLASH);

    let mut io = io::IO::new();

    while io.should_stay_open() {
        let mut instructions_this_frame = 0;
        let mut vram_changed = false;

        processor.set_keys(&io.get_keys());
        processor.update_timers();

        while instructions_this_frame < INSTRUCTIONS_PER_FRAME {
            vram_changed = vram_changed | processor.step();
            instructions_this_frame += 1;
        }

        if vram_changed {
            io.set_framebuffer(processor.get_framebuffer());
        }
        io.refresh_display();
    }
}

fn load_rom(processor: &mut Processor) {

    let path = FileDialog::new()
    .add_filter("Chip-8 Rom", &["ch8"])
    .show_open_single_file()
    .unwrap();

    let rom_path = match path {
        Some(path) => path,
        None => return,
    };

    let rom = rom::load(&rom_path).unwrap();

    processor.load_data(&rom);
}

//4A10
