use std::{fs, thread::sleep, time::Duration};

use crate::core::{
    bus::Bus,
    cartridge::Cartridge,
    cpu::{self, Cpu},
    registers::Registers,
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

        for _ in 0..cpu_cycle_info.cycle_count {
            self.bus.timer.cycle();
            self.bus.graphics.cycle();
        }

        CycleInfo { cpu_cycle_info }
    }

    pub fn get_pixels(&self) -> [u8; 144 * 160] {
        self.bus.graphics.pixels()
    }

    pub fn get_registers(&self) -> Registers {
        self.cpu.registers()
    }

    pub fn get_next_opcode(&mut self) -> ([u8; 3], u16) {
        self.cpu.get_next_opcode(&mut self.bus)
    }

    pub fn frame_ready(&self) -> bool {
        self.bus.graphics.stat & 0b00000011 == 0b00000001
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CycleInfo {
    pub cpu_cycle_info: cpu::CycleInfo,
}
