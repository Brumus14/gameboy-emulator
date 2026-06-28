use crate::{bus::Bus, cartridge::Cartridge, cpu::Cpu};

pub struct Gameboy {
    cpu: Cpu,
    bus: Bus,
}

impl Gameboy {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            bus: Bus::new(),
        }
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.bus.cartridge = Some(cartridge);
    }

    pub fn unload_cartridge(&mut self) {
        self.bus.cartridge = None;
    }

    pub fn cycle(&mut self) {
        self.cpu.cycle(&mut self.bus);
    }
}
