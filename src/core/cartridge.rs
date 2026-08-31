use std::{fs, io};

use crate::core::mbc::{Mbc, Mbc1};

pub struct Cartridge {
    pub title: String,
    pub 
    pub rom: Vec<u8>,
    ram: Vec<u8>,
    mbc: Option<Box<dyn Mbc>>,
}

impl Cartridge {
    pub fn from_file(file_path: &str) -> io::Result<Self> {
        let rom = fs::read(file_path)?;

        let (mbc, has_ram): (Option<Box<dyn Mbc>>, bool) = match rom[0x0147] {
            0x00 => (None, false),
            0x01 => (Some(Box::new(Mbc1::new())), false),
            0x02 => (Some(Box::new(Mbc1::new())), true),
            0x03 => (Some(Box::new(Mbc1::new())), true),
            _ => unreachable!(),
        };

        println!("{}", rom[0x0147]);
        let ram_bank_count = if has_ram {
            match rom[0x0149] {
                0x00 => 0,
                0x01 => 0,
                0x02 => 1,
                0x03 => 4,
                0x04 => 16,
                0x05 => 8,
                _ => unreachable!(),
            }
        } else {
            0
        };

        Ok(Self {
            rom,
            ram: vec![0; ram_bank_count * 8192],
            mbc,
        })
    }

    pub fn read(&mut self, address: u16) -> u8 {
        if let Some(mbc) = &mut self.mbc {
            mbc.read(&self.rom, &self.ram, address)
        } else {
            match address {
                0x0000..0x8000 => self.rom[address as usize],
                0xA000..0xC000 => *self.ram.get((address - 0xA000) as usize).unwrap_or(&0xFF),
                _ => unreachable!(),
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        if let Some(mbc) = &mut self.mbc {
            mbc.write(&self.rom, &mut self.ram, address, value);
        } else {
            match address {
                0x0000..0xA000 => (),
                0xA000..0xC000 => {
                    if let Some(v) = self.ram.get_mut((address - 0xA000) as usize) {
                        *v = value;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
