use crate::{memory::Memory, registers::Registers};

pub struct Cpu {
    registers: Registers,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
        }
    }

    pub fn execute(&self, memory: &Memory) {
        let opcode = memory.read(self.registers.pc);
    }
}
