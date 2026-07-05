use std::{thread::sleep, time::Duration};

use crate::core::{
    bus::Bus,
    cartridge::Cartridge,
    cpu::{self, Cpu},
};

pub struct Gameboy {
    cpu: Cpu,
    bus: Bus,
    cycle_count: u64,
}

impl Gameboy {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            bus: Bus::new(),
            cycle_count: 0,
        }
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.bus.cartridge = Some(cartridge);
    }

    pub fn unload_cartridge(&mut self) {
        self.bus.cartridge = None;
    }

    pub fn cycle(&mut self) -> CycleInfo {
        let cpu_cycle_info = self.cpu.cycle(&mut self.bus);
        self.bus.timer.cycle();
        self.bus.graphics.cycle();

        CycleInfo { cpu_cycle_info }
    }

    pub fn get_pixels(&self) -> [u8; 144 * 160] {
        self.bus.graphics.get_pixels()
    }
}

#[derive(Debug)]
pub struct CycleInfo {
    cpu_cycle_info: cpu::CycleInfo,
}
