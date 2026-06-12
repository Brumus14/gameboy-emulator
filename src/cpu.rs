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

    pub fn decode_instruction(&self, memory: &Memory) -> Instruction {
        let opcode = memory.read(self.registers.pc);
        Instruction::Nop
    }
}

#[derive(Debug)]
pub enum Instruction {
    Nop,
    LD16,
}
