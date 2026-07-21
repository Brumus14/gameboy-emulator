use std::{fs, io};

use crate::core::mbc::{Mbc, Mbc1};

pub struct Cartridge {
    pub rom: Vec<u8>,
    ram: Option<Vec<u8>>,
    mbc: Option<Box<dyn Mbc>>,
}

impl Cartridge {
    pub fn from_file(file_path: &str) -> io::Result<Self> {
        let rom = fs::read(file_path)?;

        let mbc: Option<Box<dyn Mbc>> = match rom[0x147] {
            0x01 => Some(Box::new(Mbc1::new())),
            _ => None,
        };

        Ok(Self {
            rom,
            ram: None,
            mbc,
        })
    }

    pub fn read(&mut self, address: u16) -> u8 {
        if let Some(mbc) = &mut self.mbc {
            mbc.read(&self.rom, &self.ram, address)
        } else {
            match address {
                0x0000..0x8000 => self.rom[address as usize],
                0xA000..0xC000 => {
                    if let Some(ram) = &self.ram {
                        ram[(address - 0xA000) as usize]
                    } else {
                        0xFF
                    }
                }
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
                    if let Some(ram) = &mut self.ram {
                        ram[(address - 0xA000) as usize] = value;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
