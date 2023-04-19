use chip8_emulator::{rom, processor, display};
use clap::Parser;
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("rom.ch8"))]
    rom: String,
}

fn main() {
    let args = Args::parse();
    let rom = rom::load(&args.rom).unwrap();

    let mut processor = processor::Processor::new();

    processor.load_data(&rom);
    let mut display = display::Display::new();

    while display.should_stay_open() {
        processor.step();
        display.refresh(processor.get_framebuffer());
        thread::sleep(Duration::from_millis(2));
    }
}
