use std::fs;

use crate::core::{
    audio::Apu, cartridge::Cartridge, graphics::Graphics, interrupts::Interrupts, joypad::Joypad,
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
    pub boot_rom: [u8; 256],
    boot_rom_mapped: bool,
    pub wram: [u8; 8192],
    pub hram: [u8; 127],
    pub cartridge: Option<Cartridge>,
    pub graphics: Graphics,
    pub interrupts: Interrupts,
    pub joypad: Joypad,
    pub timer: Timer,
    pub serial: Serial,
    pub audio: Apu,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            boot_rom: fs::read("res/rom/DMG_ROM.bin")
                .unwrap()
                .as_slice()
                .try_into()
                .unwrap(),
            boot_rom_mapped: true,
            wram: [0; 8192],
            hram: [0; 127],
            cartridge: None,
            graphics: Graphics::new(),
            interrupts: Interrupts::new(),
            joypad: Joypad::new(),
            timer: Timer::new(),
            serial: Serial::new(),
            audio: Apu::new(),
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        if self.boot_rom_mapped && address < 0x100 {
            return self.boot_rom[address as usize];
        }

        match address {
            0x0000..0x8000 => {
                if let Some(cartridge) = &mut self.cartridge {
                    cartridge.read(address)
                } else {
                    0
                }
            }
            0x8000..0xA000 => self.graphics.read(address),
            0xA000..0xC000 => {
                if let Some(cartridge) = &mut self.cartridge {
                    cartridge.read(address)
                } else {
                    0
                }
            }
            0xC000..0xE000 => self.wram[(address - 0xC000) as usize],
            0xE000..0xFE00 => self.wram[(address - 0xE000) as usize],
            0xFE00..0xFEA0 => self.graphics.read(address),
            0xFEA0..0xFF00 => 0xFF, // TODO: might be different
            0xFF00..0xFF80 => self.read_io(address),
            0xFF80..0xFFFF => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.interrupts.read(address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        // Should this be removed?
        if self.boot_rom_mapped && address < 0x100 {
            return;
        }

        match address {
            0x0000..0x8000 => {
                if let Some(cartridge) = &mut self.cartridge {
                    cartridge.write(address, value);
                }
            }
            0x8000..0xA000 => self.graphics.write(address, value),
            0xA000..0xC000 => {
                if let Some(cartridge) = &mut self.cartridge {
                    cartridge.write(address, value)
                }
            }
            0xC000..0xE000 => self.wram[(address - 0xC000) as usize] = value,
            0xE000..0xFE00 => self.wram[(address - 0xE000) as usize] = value,
            0xFE00..0xFEA0 => self.graphics.write(address, value),
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
            0xFF40..=0xFF4B => self.graphics.read(address),
            // 0xFF50 => ,
            0xFF70..=0xFFFF => unreachable!(),
            _ => 0,
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
            0xFF40..=0xFF45 => self.graphics.write(address, value),
            0xFF46 => self.oam_dma_transfer(value),
            0xFF47..=0xFF4B => self.graphics.write(address, value),
            0xFF50 => self.boot_rom_mapped = false, // Boot ROM mapping
            0xFF80..=0xFFFF => unreachable!(),
            _ => (),
        }
    }

    fn oam_dma_transfer(&mut self, value: u8) {
        let start = (value as u16) << 8;

        for i in 0..160 {
            let value = self.read(start + i);
            self.write(0xFE00 + i, value);
        }
    }
}
