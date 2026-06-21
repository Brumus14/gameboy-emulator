use std::intrinsics::unreachable;

pub trait Mbc {
    fn read(&mut self, rom: &Vec<u8>, address: u16) -> u8;
    fn write(&mut self, rom: &Vec<u8>, address: u16, value: u8);
    // fn read_bank_0(&self, rom: &Vec<u8>, offset: u16);
    // fn write_bank_0(&mut self, rom: &Vec<u8>, offset: u16);
    // fn read_bank_n(&self, rom: &Vec<u8>, offset: u16);
    // fn write_bank_n(&mut self, rom: &Vec<u8>, offset: u16);
}

pub struct Mbc1 {
    ram_enable: bool,
    rom_bank_number: u8,
    ram_bank_number: u8,
    banking_mode: u8,
}

impl Mbc1 {
    fn new() -> Self {
        Self {
            ram_enable: false,
            rom_bank_number: 0,
            ram_bank_number: 0,
            banking_mode: 0,
        }
    }
}

impl Mbc for Mbc1 {
    fn read(&mut self, rom: &Vec<u8>, address: u16) -> u8 {
        match address {
            0x0000..0x4000 => rom[address as usize],
            0x4000..0x8000 => {
                let address = address;
                rom[address as usize]
            }
            _ => unreachable!(),
        }
    }

    fn write(&mut self, rom: &Vec<u8>, address: u16, value: u8) {
        match address {
            0x0000..0x2000 => {
                if value & 0xF == 0xA {
                    self.ram_enable = true;
                } else {
                    self.ram_enable = false;
                }
            }
            0x2000..0x4000 => {
                self.rom_bank_number = value & 0b00011111;
            }
            0x4000..0x6000 => {
                self.ram_bank_number = value & 0b00000011;
            }
            0x6000..0x8000 => {
                self.banking_mode = value & 1;
            }
            _ => unreachable!(),
        }
    }
}
