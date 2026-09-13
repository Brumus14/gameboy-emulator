pub trait Mbc {
    fn read(&mut self, rom: &Vec<u8>, ram: &Option<Vec<u8>>, address: u16) -> u8;
    fn write(&mut self, rom: &Vec<u8>, ram: &mut Option<Vec<u8>>, address: u16, value: u8);
}

pub struct Mbc1 {
    ram_enabled: bool,
    rom_bank_number: u16,
    rom_bank_count: u16,
    ram_bank_number: u8,
    ram_bank_count: u8,
    banking_mode: u8,
}

impl Mbc1 {
    pub fn new(rom_bank_count: u16, ram_bank_count: u8) -> Self {
        Self {
            ram_enabled: false,
            rom_bank_number: 0,
            ram_bank_number: 0,
            banking_mode: 0,
            rom_bank_count,
            ram_bank_count,
        }
    }
}

impl Mbc for Mbc1 {
    fn read(&mut self, rom: &Vec<u8>, ram: &Option<Vec<u8>>, address: u16) -> u8 {
        match address {
            0x0000..0x4000 => {
                if self.banking_mode == 1 && self.rom_bank_count >= 64 {
                    rom[(self.ram_bank_number as usize) << 19 | address as usize]
                } else {
                    rom[address as usize]
                }
            }
            0x4000..0x8000 => {
                let rom_bank = self.rom_bank_number.max(1);
                let bank_number = if self.rom_bank_count <= 32 {
                    rom_bank % self.rom_bank_count
                } else {
                    ((self.ram_bank_number as u16) << 5) | (rom_bank & 0x00001111)
                };

                rom[((bank_number as usize) << 14) | address as usize]
            }
            0xA000..0xC000 => {
                if self.ram_enabled {
                    ram.as_ref().unwrap()[(address - 0xA000) as usize]
                } else {
                    0xFF
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
                self.rom_bank_number = (value & 0b00011111) as u16;
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
                    if let Some(v) = ram.get_mut((address - 0xA000) as usize) {
                        *v = value;
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
