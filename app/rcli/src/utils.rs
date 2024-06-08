use anyhow::Result;
use std::{fs::File, io::Read};

pub fn get_reader(input: &str) -> Result<Box<dyn Read>> {
    let reader: Box<dyn Read> = if input == "-" {
        Box::new(std::io::stdin())
    } else {
        Box::new(File::open(input)?)
    };
    Ok(reader)
}

pub fn get_content(input: &str) -> Result<Vec<u8>> {
    let mut reader = get_reader(input)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn trim_trailing_newline(vec: Vec<u8>) -> Vec<u8> {
    let mut trimmed = vec;
    // if windows
    if trimmed.len() >= 2 && trimmed[trimmed.len() - 2..] == [0xD, 0xA] {
        trimmed.pop();
        trimmed.pop();
    } else if let Some(&0xA) = trimmed.last() {
        trimmed.pop();
    }
    trimmed
}
