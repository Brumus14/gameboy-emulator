use std::{fs, thread::sleep, time::Duration};

use crate::core::{
    bus::Bus,
    cartridge::Cartridge,
    cpu::{self, Cpu},
    joypad::JoypadState,
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

    pub fn restart(&mut self) {
        let cartridge = self.bus.cartridge.take();

        self.cpu = Cpu::new();
        self.bus = Bus::new();
        self.cycle_count = 0;

        self.bus.cartridge = cartridge;
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.bus.cartridge = Some(cartridge);
    }

    pub fn unload_cartridge(&mut self) {
        self.bus.cartridge = None;
    }

    // TODO: Should be cycling simultaneously
    pub fn cycle(&mut self) -> CycleInfo {
        let cpu_cycle_info = self.cpu.cycle(&mut self.bus);

        let c_cycle_count = if let Some(cycle_info) = cpu_cycle_info {
            cycle_info.cycle_count
        } else {
            1
        } * 4;

        for _ in 0..c_cycle_count {
            let timer_interrupt = self.bus.timer.cycle();

            if timer_interrupt {
                self.bus.interrupts.flag |= 1 << 2;
            }

            let graphics_interrupt = self.bus.graphics.cycle();

            if graphics_interrupt {
                self.bus.interrupts.flag |= 1;
            }
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
        self.bus.graphics.get_mode() == 1
    }

    pub fn update_joypad_state(&mut self, state: JoypadState) {
        self.bus.joypad.state = state;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CycleInfo {
    pub cpu_cycle_info: Option<cpu::CycleInfo>,
}
