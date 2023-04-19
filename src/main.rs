use chip8_emulator::{rom, processor, io};
use clap::Parser;
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help="The rom to run" , default_value_t = String::from("rom.ch8"))]
    rom: String,
}

fn main() {
    let args = Args::parse();
    let rom = rom::load(&args.rom).unwrap();

    let mut processor = processor::Processor::new();

    processor.load_data(&rom);
    let mut io = io::IO::new();

    while io.should_stay_open() {
        processor.set_keys(&io.get_keys());
        processor.step();
        io.refresh(processor.get_framebuffer());
        thread::sleep(Duration::from_secs_f64(1f64/1000f64));
    }
}
