#![windows_subsystem = "windows"]

use chip8_emulator::{io::{self, MenuAction::OpenFile, MenuAction::Reset, MenuAction::SetColors}, rom, processor::{self, Processor}, splash, config};
use native_dialog::FileDialog;

const INSTRUCTIONS_PER_FRAME: u32 = 15;

fn main() {
    let mut config = config::Config::build().unwrap();

    let mut processor = processor::Processor::new();

    processor.load_data(&splash::SPLASH);

    let mut io = io::IO::new(config.on_color, config.off_color,  config.get_palettes());

    while io.should_stay_open() {
        if let Some(action) = io.get_current_menu_action() {
            match action {
                OpenFile => load_rom(&mut processor),
                Reset => processor.reset(),
                SetColors(on_color, off_color) => {
                    config.on_color = *on_color;
                    config.off_color = *off_color;
                    match config.save_to_file() {
                        Ok(()) => println!("Saved configuration succesfully"),
                        Err(e) => println!("ERROR Failed to save configuration file: {e}"),
                    }
                }
            }
        }

        let mut instructions_this_frame = 0;
        let mut vram_changed = false;

        processor.set_keys(&io.get_keys());
        processor.update_timers();

        while instructions_this_frame < INSTRUCTIONS_PER_FRAME && !vram_changed {
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

    println!("{}", rom_path.file_name().unwrap().to_str().unwrap());

    processor.load_data(&rom);
    processor.reset();
}

//4A10
