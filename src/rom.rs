use std::io::{self, Read, BufReader};
use std::fs::File;

pub fn load() -> io::Result<Vec<u8>> {
    let f = File::open("rom.ch8")?;
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer)?;

    Ok(buffer)
}