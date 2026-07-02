use std::{thread::sleep, time::Duration};

use crate::core::{bus::Bus, cartridge::Cartridge, cpu::Cpu};

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

    pub fn on(&mut self) {
        loop {
            self.cycle();
            sleep(Duration::from_millis(100));
        }
    }

    fn cycle(&mut self) {
        self.cpu.cycle(&mut self.bus);
        self.bus.timer.tick();
    }
}
