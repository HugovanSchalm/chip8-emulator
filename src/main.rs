use chip8_emulator::{io, processor, rom};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help="The rom to run" , default_value_t = String::from("rom.ch8"))]
    rom: String,
}

const INSTRUCTIONS_PER_FRAME: u32 = 20;

fn main() {
    let args = Args::parse();
    let rom = rom::load(&args.rom).unwrap();

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
        io.refresh();
    }
}

//4A10
