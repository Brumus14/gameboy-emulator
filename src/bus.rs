use crate::{
    audio::Audio, cartridge::Cartridge, interrupts::Interrupts, joypad::Joypad, ppu::Ppu,
    serial::Serial, timer::Timer,
};

#[derive(Debug)]
enum Region {
    ExternalRom,
    VideoRam,
    ExternalRam,
    WorkRam,
    EchoArea,
    ObjectAttributeMemory,
    Unused,
    IoRegisters,
    HighRam,
    InterruptEnableRegister,
}

pub struct Bus {
    pub wram: [u8; 8192],
    pub hram: [u8; 127],
    pub cartridge: Option<Cartridge>,
    pub ppu: Ppu,
    pub interrupts: Interrupts,
    pub joypad: Joypad,
    pub timer: Timer,
    pub serial: Serial,
    pub audio: Audio,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            wram: [0; 8192],
            hram: [0; 127],
            cartridge: None,
            ppu: Ppu::new(),
            interrupts: Interrupts::new(),
            joypad: Joypad::new(),
            timer: Timer::new(),
            serial: Serial::new(),
            audio: Audio::new(),
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..0x8000 => {
                if let Some(cartridge) = &mut self.cartridge {
                    cartridge.read(address)
                } else {
                    0xFF
                }
            }
            0x8000..0xA000 => self.ppu.read(address),
            0xA000..0xC000 => {
                if let Some(cartridge) = &mut self.cartridge {
                    cartridge.read(address)
                } else {
                    0xFF
                }
            }
            0xC000..0xE000 => self.wram[(address - 0xC000) as usize],
            0xE000..0xFE00 => self.wram[(address - 0xE000) as usize],
            0xFE00..0xFEA0 => self.ppu.read(address),
            0xFEA0..0xFF00 => 0xFF, // TODO: might be different
            0xFF00..0xFF80 => self.read_io(address),
            0xFF80..0xFFFF => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.interrupts.read(address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..0x8000 => {
                if let Some(cartridge) = &mut self.cartridge {
                    cartridge.write(address, value);
                }
            }
            0x8000..0xA000 => self.ppu.write(address, value),
            0xA000..0xC000 => {
                if let Some(cartridge) = &mut self.cartridge {
                    cartridge.write(address, value)
                }
            }
            0xC000..0xE000 => self.wram[(address - 0xC000) as usize] = value,
            0xE000..0xFE00 => self.wram[(address - 0xE000) as usize] = value,
            0xFE00..0xFEA0 => self.ppu.write(address, value),
            0xFEA0..0xFF00 => (), // TODO: might be different
            0xFF00..0xFF80 => self.write_io(address, value),
            0xFF80..0xFFFF => self.hram[(address - 0xFF80) as usize] = value,
            0xFFFF => self.interrupts.write(address, value),
        }
    }

    fn read_io(&self, address: u16) -> u8 {
        match address {
            0x0000..0xFF00 => unreachable!(),
            0xFF00 => self.joypad.read(),
            0xFF01..=0xFF02 => self.serial.read(address),
            0xFF04..=0xFF07 => self.timer.read(address),
            0xFF0F => self.interrupts.read(address),
            0xFF10..=0xFF3F => self.audio.read(address),
            0xFF40..=0xFF4B => self.ppu.read(address),
            0xFF50 => todo!(),
            0xFF70..=0xFFFF => unreachable!(),
            _ => 0xFF,
        }
    }

    fn write_io(&mut self, address: u16, value: u8) {
        match address {
            0x0000..0xFF00 => unreachable!(),
            0xFF00 => self.joypad.write(value),
            0xFF01..=0xFF02 => self.serial.write(address, value),
            0xFF04..=0xFF07 => self.timer.write(address, value),
            0xFF0F => self.interrupts.write(address, value),
            0xFF10..=0xFF3F => self.audio.write(address, value),
            0xFF40..=0xFF4B => self.ppu.write(address, value),
            0xFF50 => todo!(),
            0xFF70..=0xFFFF => unreachable!(),
            _ => (),
        }
    }
}
