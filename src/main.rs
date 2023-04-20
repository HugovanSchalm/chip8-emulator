use chip8_emulator::{io, processor, rom};
use clap::Parser;
use native_dialog::FileDialog;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(required = false, short, long, help = "The rom to run")]
    rom: Option<String>,
}

const INSTRUCTIONS_PER_FRAME: u32 = 15;

fn main() {
    let args = Args::parse();

    let rom_path;
    if args.rom.is_none() {
        let path = FileDialog::new()
            .add_filter("Chip-8 Rom", &["ch8"])
            .show_open_single_file()
            .unwrap();

        rom_path = match path {
            Some(path) => path,
            None => return,
        };
    } else {
        rom_path = PathBuf::from(args.rom.unwrap());
    }

    let rom = rom::load(&rom_path).unwrap();

    let mut processor = processor::Processor::new();

    processor.load_data(&rom);
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

//4A10
