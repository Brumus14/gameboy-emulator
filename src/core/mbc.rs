pub trait Mbc {
    fn read(&mut self, rom: &Vec<u8>, ram: &Option<Vec<u8>>, address: u16) -> u8;
    fn write(&mut self, rom: &Vec<u8>, ram: &mut Option<Vec<u8>>, address: u16, value: u8);
}

pub struct Mbc1 {
    ram_enabled: bool,
    rom_bank_number: u8,
    ram_bank_number: u8,
    banking_mode: u8,
}

impl Mbc1 {
    pub fn new() -> Self {
        Self {
            ram_enabled: false,
            rom_bank_number: 0,
            ram_bank_number: 0,
            banking_mode: 0,
        }
    }
}

impl Mbc for Mbc1 {
    fn read(&mut self, rom: &Vec<u8>, ram: &Option<Vec<u8>>, address: u16) -> u8 {
        match address {
            0x0000..0x4000 => rom[address as usize],
            0x4000..0x8000 => {
                let bank_number = self.rom_bank_number.min(1);
                rom[((bank_number as u16) * 0x4000 + address) as usize]
            }
            0xA000..0xC000 => {
                if self.ram_enabled
                    && let Some(ram) = ram
                {
                    ram[(address - 0xA000) as usize]
                } else {
                    0 // maybe not?
                }
            }
            _ => unreachable!(),
        }
    }

    fn write(&mut self, rom: &Vec<u8>, ram: &mut Option<Vec<u8>>, address: u16, value: u8) {
        match address {
            0x0000..0x2000 => {
                if value & 0xF == 0xA {
                    self.ram_enabled = true;
                } else {
                    self.ram_enabled = false;
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
            0xA000..0xC000 => {
                if self.ram_enabled
                    && let Some(ram) = ram
                {
                    ram[(address - 0xA000) as usize] = value;
                }
            }
            _ => unreachable!(),
        }
    }
}
