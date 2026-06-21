use crate::{cpu::Cpu, memory::Memory};

pub struct Gameboy {
    cpu: Cpu,
    memory: Memory,
}

impl Gameboy {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            memory: Memory::new(),
        }
    }

    pub fn load_rom(&mut self) {}

    pub fn cycle(&mut self) {
        let instruction = self.cpu.cycle(&mut self.memory);
        println!("{:?}", instruction);
    }
}
