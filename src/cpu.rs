use crate::registers::Registers;

pub struct CPU {
    registers: Registers,
}

impl CPU {
    pub fn decode_instruction() {}
}

enum Instruction {
    Nop,
    LD16,
}
