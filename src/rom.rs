use std::io::{self, Read, BufReader};
use std::fs::File;
use std::path::PathBuf;

pub fn load(filename: &PathBuf) -> io::Result<Vec<u8>> {
    let f = File::open(filename)?;
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer)?;

    Ok(buffer)
}