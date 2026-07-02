use crate::core::{
    bus::Bus,
    registers::{Flag, Register8, Register16, Registers},
};

#[derive(Clone, Copy)]
pub enum OperandType {
    R8,
    R16,
    R16stk,
    R16mem,
    Cond,
    B3,
    Tgt3,
}

#[derive(Clone, Copy)]
pub enum R8 {
    B,
    C,
    D,
    E,
    H,
    L,
    MemoryHL,
    A,
}

#[derive(Clone, Copy)]
pub enum R16 {
    BC,
    DE,
    HL,
    SP,
}

#[derive(Clone, Copy)]
pub enum R16stk {
    BC,
    DE,
    HL,
    AF,
}

#[derive(Clone, Copy)]
pub enum R16mem {
    BC,
    DE,
    HLI,
    HLD,
}

#[derive(Clone, Copy)]
pub enum Cond {
    NZ,
    Z,
    NC,
    C,
}

pub fn decode_r8(operand: u8) -> R8 {
    match operand {
        0 => R8::B,
        1 => R8::C,
        2 => R8::D,
        3 => R8::E,
        4 => R8::H,
        5 => R8::L,
        6 => R8::MemoryHL,
        7 => R8::A,
        _ => unreachable!(),
    }
}

pub fn decode_r16(operand: u8) -> R16 {
    match operand {
        0 => R16::BC,
        1 => R16::DE,
        2 => R16::HL,
        3 => R16::SP,
        _ => unreachable!(),
    }
}

pub fn decode_r16stk(operand: u8) -> R16stk {
    match operand {
        0 => R16stk::BC,
        1 => R16stk::DE,
        2 => R16stk::HL,
        3 => R16stk::AF,
        _ => unreachable!(),
    }
}

pub fn decode_r16mem(operand: u8) -> R16mem {
    match operand {
        0 => R16mem::BC,
        1 => R16mem::DE,
        2 => R16mem::HLI,
        3 => R16mem::HLD,
        _ => unreachable!(),
    }
}

pub fn decode_cond(operand: u8) -> Cond {
    match operand {
        0 => Cond::NZ,
        1 => Cond::Z,
        2 => Cond::NC,
        3 => Cond::C,
        _ => unreachable!(),
    }
}

pub fn parse_operand(opcode: u8, start_bit_index: u8, operand_type: OperandType) -> u8 {
    let operand_size = match operand_type {
        OperandType::R8 => 3,
        OperandType::R16 => 2,
        OperandType::R16stk => 2,
        OperandType::R16mem => 2,
        OperandType::Cond => 2,
        OperandType::B3 => 3,
        OperandType::Tgt3 => 3,
    };

    (opcode >> start_bit_index) & ((1 << operand_size) - 1)
}

pub fn get_r8(operand: R8, registers: &Registers, bus: &mut Bus) -> u8 {
    match operand {
        R8::B => registers.get_register8(Register8::B),
        R8::C => registers.get_register8(Register8::C),
        R8::D => registers.get_register8(Register8::D),
        R8::E => registers.get_register8(Register8::E),
        R8::H => registers.get_register8(Register8::H),
        R8::L => registers.get_register8(Register8::L),
        R8::MemoryHL => bus.read(registers.get_register16(Register16::HL)),
        R8::A => registers.get_register8(Register8::A),
    }
}

pub fn set_r8(operand: R8, value: u8, registers: &mut Registers, bus: &mut Bus) {
    match operand {
        R8::B => registers.set_register8(Register8::B, value),
        R8::C => registers.set_register8(Register8::C, value),
        R8::D => registers.set_register8(Register8::D, value),
        R8::E => registers.set_register8(Register8::E, value),
        R8::H => registers.set_register8(Register8::H, value),
        R8::L => registers.set_register8(Register8::L, value),
        R8::MemoryHL => bus.write(registers.get_register16(Register16::HL), value),
        R8::A => registers.set_register8(Register8::A, value),
    }
}

pub fn get_r16(operand: R16, registers: &Registers) -> u16 {
    match operand {
        R16::BC => registers.get_register16(Register16::BC),
        R16::DE => registers.get_register16(Register16::DE),
        R16::HL => registers.get_register16(Register16::HL),
        R16::SP => registers.get_register16(Register16::SP),
    }
}

pub fn set_r16(operand: R16, value: u16, registers: &mut Registers) {
    match operand {
        R16::BC => registers.set_register16(Register16::BC, value),
        R16::DE => registers.set_register16(Register16::DE, value),
        R16::HL => registers.set_register16(Register16::HL, value),
        R16::SP => registers.set_register16(Register16::SP, value),
    }
}

pub fn get_r16stk(operand: R16stk, registers: &Registers) -> u16 {
    match operand {
        R16stk::BC => registers.get_register16(Register16::BC),
        R16stk::DE => registers.get_register16(Register16::DE),
        R16stk::HL => registers.get_register16(Register16::HL),
        R16stk::AF => registers.get_register16(Register16::AF),
    }
}

pub fn set_r16stk(operand: R16stk, value: u16, registers: &mut Registers) {
    match operand {
        R16stk::BC => registers.set_register16(Register16::BC, value),
        R16stk::DE => registers.set_register16(Register16::DE, value),
        R16stk::HL => registers.set_register16(Register16::HL, value),
        R16stk::AF => registers.set_register16(Register16::AF, value),
    }
}

pub fn get_r16mem(operand: R16mem, registers: &Registers) -> u16 {
    match operand {
        R16mem::BC => registers.get_register16(Register16::BC),
        R16mem::DE => registers.get_register16(Register16::DE),
        R16mem::HLI => registers.get_register16(Register16::HL),
        R16mem::HLD => registers.get_register16(Register16::HL),
    }
}

pub fn set_r16mem(operand: R16mem, value: u16, registers: &mut Registers) {
    match operand {
        R16mem::BC => registers.set_register16(Register16::BC, value),
        R16mem::DE => registers.set_register16(Register16::DE, value),
        R16mem::HLI => registers.set_register16(Register16::HL, value),
        R16mem::HLD => registers.set_register16(Register16::HL, value),
    }
}

pub fn get_cond(cond: Cond, registers: &Registers) -> bool {
    match cond {
        Cond::NZ => !registers.get_flag(Flag::Zero),
        Cond::Z => registers.get_flag(Flag::Zero),
        Cond::NC => !registers.get_flag(Flag::Carry),
        Cond::C => registers.get_flag(Flag::Carry),
    }
}
