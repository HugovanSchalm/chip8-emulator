use chip8_emulator::{rom, processor, display};
use std::thread;
use std::time::Duration;
fn main() {
    let rom = rom::load().unwrap();
    let mut processor = processor::Processor::new();
    processor.load_data(&rom);
    let mut display = display::Display::new();

    while display.should_stay_open() {
        processor.step();
        display.refresh(processor.get_framebuffer());
        thread::sleep(Duration::from_millis(2));
    }
}
