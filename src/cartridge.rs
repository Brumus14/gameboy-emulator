use std::{fs, io};

use crate::mbc::Mbc;

pub struct Cartridge {
    rom: Vec<u8>,
    mbc: Option<Box<dyn Mbc>>,
}

impl Cartridge {
    pub fn from_file(file_path: &str) -> io::Result<Self> {
        let rom = fs::read(file_path)?;

        Ok(Self { rom, mbc: None })
    }
}
