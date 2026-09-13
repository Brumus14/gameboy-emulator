use std::{
    fs,
    io::{self, Read, Seek, SeekFrom},
};

use crate::core::mbc::{Mbc, Mbc1};

pub struct Cartridge {
    type_code: u8,
    rom_size_code: u8,
    ram_size_code: u8,
    rom_bank_count: u16,
    ram_bank_count: u8,
    rom: Vec<u8>,
    ram: Option<Vec<u8>>,
    mbc: Option<Box<dyn Mbc>>,
}

impl Cartridge {
    pub fn from_file(file_path: &str) -> io::Result<Self> {
        let mut rom_file = fs::File::open(file_path)?;
        rom_file.seek(SeekFrom::Start(0x100))?;

        let mut rom_header = vec![0; 0x4F];
        rom_file.read_exact(&mut rom_header)?;

        let type_code = rom_header[0x47];
        let rom_size_code = rom_header[0x48];
        let ram_size_code = rom_header[0x49];

        let (rom_size, rom_bank_count) = match rom_size_code {
            0x00 => (32 * 1024, 2),
            0x01 => (64 * 1024, 4),
            0x02 => (128 * 1024, 8),
            0x03 => (256 * 1024, 16),
            0x04 => (512 * 1024, 32),
            0x05 => (1024 * 1024, 64),
            0x06 => (2 * 1024 * 1024, 128),
            0x07 => (4 * 1024 * 1024, 256),
            0x08 => (8 * 1024 * 1024, 512),
            _ => unreachable!(),
        };

        let ram_bank_count: u8 = match ram_size_code {
            0x00 => 0,
            0x02 => 1,
            0x03 => 4,
            0x04 => 16,
            0x05 => 8,
            _ => unreachable!(),
        };

        let (mbc, has_ram): (Option<Box<dyn Mbc>>, bool) = match type_code {
            0x00 => (None, false),
            0x01 => (
                Some(Box::new(Mbc1::new(rom_bank_count, ram_bank_count))),
                false,
            ),
            0x02 => (
                Some(Box::new(Mbc1::new(rom_bank_count, ram_bank_count))),
                true,
            ),
            0x03 => (
                Some(Box::new(Mbc1::new(rom_bank_count, ram_bank_count))),
                true,
            ),
            _ => unreachable!(),
        };

        let ram: Option<Vec<u8>> = if has_ram {
            Some(vec![0; (ram_bank_count as usize) * 8192])
        } else {
            None
        };

        let mut rom = vec![0; rom_size];
        rom_file.seek(SeekFrom::Start(0))?;
        rom_file.read(&mut rom)?;

        Ok(Self {
            type_code,
            rom_size_code,
            rom_bank_count,
            ram_size_code,
            ram_bank_count,
            rom,
            ram,
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
                        *ram.get((address - 0xA000) as usize).unwrap_or(&0xFF)
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
                        if let Some(v) = ram.get_mut((address - 0xA000) as usize) {
                            *v = value;
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
