use crate::opcodes::{
    Cond, OperandType, R8, R16, R16mem, decode_cond, decode_r8, decode_r16, decode_r16mem,
    get_cond, get_r8, get_r16, get_r16mem, parse_operand, set_r8, set_r16,
};
use crate::registers::{Flag, Register8, Register16};

use crate::{memory::Memory, registers::Registers};

pub enum Condition {
    NotZero,
    Zero,
    NotCarry,
    Carry,
}

pub struct Cpu {
    registers: Registers,
    interrupt_master_enable: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            interrupt_master_enable: false,
        }
    }

    fn fetch(&mut self, memory: &Memory) -> u8 {
        let pc = self.registers.get_register16(Register16::PC);
        self.registers.set_register16(Register16::PC, pc + 1);
        memory.read(pc)
    }

    fn fetch_16(&mut self, memory: &Memory) -> u16 {
        // Little-endian order
        (self.fetch(memory) as u16) | ((self.fetch(memory) as u16) << 8)
    }

    pub fn cycle(&mut self, memory: &mut Memory) {
        let opcode = self.fetch(memory);

        if opcode == 0b00000000 {
        } else if opcode & 0b11001111 == 0b00000001 {
            let r16 = decode_r16(parse_operand(opcode, 4, OperandType::R16));
            self.ld_r16_imm16(r16, memory);
        } else if opcode & 0b11001111 == 0b00000010 {
            let r16mem = decode_r16mem(parse_operand(opcode, 4, OperandType::R16mem));
            self.ld_r16mem_a(r16mem, memory);
        } else if opcode & 0b11001111 == 0b00001010 {
            let r16mem = decode_r16mem(parse_operand(opcode, 4, OperandType::R16mem));
            self.ld_a_r16mem(r16mem, memory);
        } else if opcode == 0b00001000 {
            self.ld_imm16_sp(memory);
        } else if opcode & 0b11001111 == 0b00000011 {
            let r16 = decode_r16(parse_operand(opcode, 4, OperandType::R16));
            self.inc_r16(r16);
        } else if opcode & 0b11001111 == 0b00001011 {
            let r16 = decode_r16(parse_operand(opcode, 4, OperandType::R16));
            self.dec_r16(r16);
        } else if opcode & 0b11001111 == 0b00001001 {
            let r16 = decode_r16(parse_operand(opcode, 4, OperandType::R16));
            self.add_hl_r16(r16);
        } else if opcode & 0b11000111 == 0b00000100 {
            let r8 = decode_r8(parse_operand(opcode, 3, OperandType::R8));
            self.inc_r8(r8, memory);
        } else if opcode & 0b11000111 == 0b00000101 {
            let r8 = decode_r8(parse_operand(opcode, 3, OperandType::R8));
            self.dec_r8(r8, memory);
        } else if opcode & 0b11000111 == 0b00000110 {
            let r8 = decode_r8(parse_operand(opcode, 3, OperandType::R8));
            self.ld_r8_imm8(r8, memory);
        } else if opcode == 0b00000111 {
            self.rlca();
        } else if opcode == 0b00001111 {
            self.rrca();
        } else if opcode == 0b00010111 {
            self.rla();
        } else if opcode == 0b00011111 {
            self.rra();
        } else if opcode == 0b00100111 {
            self.daa();
        } else if opcode == 0b00101111 {
            self.cpl();
        } else if opcode == 0b00110111 {
            self.scf();
        } else if opcode == 0b00111111 {
            self.ccf();
        } else if opcode == 0b00011000 {
            self.jr_imm8(memory);
        } else if opcode & 0b11100111 == 0b00100000 {
            let cond = decode_cond(parse_operand(opcode, 3, OperandType::Cond));
            self.jr_cond_imm8(cond, memory);
        } else if opcode == 0b00010000 {
            todo!()
        } else if opcode & 0b11100111 == 0b00100000 && opcode != 0b01110110 {
            let r8a = decode_r8(parse_operand(opcode, 0, OperandType::R8));
            let r8b = decode_r8(parse_operand(opcode, 3, OperandType::R8));
            self.ld_r8_r8(r8a, r8b, memory);
        } else if opcode == 0b01110110 {
            todo!()
        } else if opcode & 0b11111000 == 0b10000000 {
            let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
            self.add_a_r8(r8, memory);
        }
    }

    fn ld_r16_imm16(&mut self, r16: R16, memory: &Memory) {
        let value = self.fetch_16(memory);
        set_r16(r16, value, &mut self.registers);
    }

    fn ld_r16mem_a(&mut self, r16mem: R16mem, memory: &mut Memory) {
        let address = get_r16mem(r16mem, &self.registers);
        memory.write(address, self.registers.a);
    }

    fn ld_a_r16mem(&mut self, r16mem: R16mem, memory: &mut Memory) {
        let address = get_r16mem(r16mem, &self.registers);
        self.registers.a = memory.read(address);
    }

    fn ld_imm16_sp(&mut self, memory: &mut Memory) {
        let address = self.fetch_16(memory);
        let value = self.registers.sp;
        memory.write(address, (value & 0xFF) as u8);
        memory.write(address + 1, (value >> 8) as u8);
    }

    fn inc_r16(&mut self, r16: R16) {
        let value = get_r16(r16, &self.registers).wrapping_add(1);
        set_r16(r16, value, &mut self.registers);
    }

    fn dec_r16(&mut self, r16: R16) {
        let value = get_r16(r16, &self.registers).wrapping_sub(1);
        set_r16(r16, value, &mut self.registers);
    }

    fn add_hl_r16(&mut self, r16: R16) {
        let hl = self.registers.get_register16(Register16::HL);
        let value = get_r16(r16, &self.registers);

        let (result, carry) = hl.overflowing_add(value);
        set_r16(r16, result, &mut self.registers);

        let half_carry = (hl & 0xFFF) + (value & 0xFFF) > 0xFFF;

        self.registers.set_flag(Flag::Subtraction, false);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
        self.registers.set_flag(Flag::Carry, carry);
    }

    fn inc_r8(&mut self, r8: R8, memory: &mut Memory) {
        let value = get_r8(r8, &mut self.registers, memory);
        let result = value.wrapping_add(1);
        set_r8(r8, result, &mut self.registers, memory);

        let half_carry = (value & 0xF) + 1 > 0xF;

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Subtraction, false);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
    }

    fn dec_r8(&mut self, r8: R8, memory: &mut Memory) {
        let value = get_r8(r8, &self.registers, memory);
        let result = value.wrapping_sub(1);
        set_r8(r8, result, &mut self.registers, memory);

        let half_carry = value & 0xF == 0;

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Subtraction, true);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
    }

    fn ld_r8_imm8(&mut self, r8: R8, memory: &mut Memory) {
        let value = self.fetch(memory);
        set_r8(r8, value, &mut self.registers, memory);
    }

    fn rlca(&mut self) {
        let value = self.registers.get_register8(Register8::A);
        let result = value.rotate_left(1);
        self.registers.set_register8(Register8::A, result);

        self.registers.set_flag(Flag::Zero, false);
        self.registers.set_flag(Flag::Subtraction, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, result & 1 == 1);
    }

    fn rrca(&mut self) {
        let value = self.registers.get_register8(Register8::A);
        let result = value.rotate_right(1);

        self.registers.set_flag(Flag::Zero, false);
        self.registers.set_flag(Flag::Subtraction, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, result >> 7 == 1);
    }

    fn rla(&mut self) {
        let value = self.registers.get_register8(Register8::A);
        let mut result = value.rotate_left(1);
        let carry = result & 1 == 1;

        if carry {
            result |= 1;
        } else {
            result &= !1;
        }

        self.registers.set_register8(Register8::A, result);

        self.registers.set_flag(Flag::Zero, false);
        self.registers.set_flag(Flag::Subtraction, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, carry);
    }

    fn rra(&mut self) {
        let value = self.registers.get_register8(Register8::A);
        let mut result = value.rotate_right(1);
        let carry = result >> 7 == 1;

        if carry {
            result |= 1 << 7;
        } else {
            result &= !(1 << 7);
        }

        self.registers.set_register8(Register8::A, result);

        self.registers.set_flag(Flag::Zero, false);
        self.registers.set_flag(Flag::Subtraction, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, carry);
    }

    fn daa(&mut self) {
        let mut adjustment: u8 = 0;

        let a = self.registers.get_register8(Register8::A);
        let half_carry = self.registers.get_flag(Flag::HalfCarry);
        let carry = self.registers.get_flag(Flag::Carry);

        if self.registers.get_flag(Flag::Subtraction) {
            if half_carry {
                adjustment += 0x6;
            }

            if carry {
                adjustment += 0x60;
            }

            self.registers
                .set_register8(Register8::A, a.wrapping_sub(adjustment));
        } else {
            if half_carry || a & 0xF > 0x9 {
                adjustment += 0x6;
            }

            if carry || a > 0x99 {
                adjustment += 0x60;
                self.registers.set_flag(Flag::Carry, true);
            }

            self.registers
                .set_register8(Register8::A, a.wrapping_add(adjustment));
        }

        self.registers.set_flag(
            Flag::HalfCarry,
            self.registers.get_register8(Register8::A) == 0,
        );
        self.registers.set_flag(Flag::HalfCarry, false);
    }

    fn cpl(&mut self) {
        let a = self.registers.get_register8(Register8::A);
        self.registers.set_register8(Register8::A, !a);

        self.registers.set_flag(Flag::Subtraction, true);
        self.registers.set_flag(Flag::HalfCarry, true);
    }

    fn scf(&mut self) {
        self.registers.set_flag(Flag::Subtraction, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, true);
    }

    fn ccf(&mut self) {
        self.registers.set_flag(Flag::Subtraction, false);
        self.registers.set_flag(Flag::HalfCarry, false);

        let carry = self.registers.get_flag(Flag::Carry);
        self.registers.set_flag(Flag::Carry, !carry);
    }

    fn jr_imm8(&mut self, memory: &Memory) {
        let pc = self.registers.get_register16(Register16::PC);
        let offset = self.fetch(memory) as i8 as i16;

        self.registers
            .set_register16(Register16::PC, pc.wrapping_add_signed(offset));
    }

    fn jr_cond_imm8(&mut self, cond: Cond, memory: &mut Memory) {
        if !get_cond(cond, &self.registers) {
            return;
        }

        self.jr_imm8(memory);
    }

    fn stop(&mut self, memory: &mut Memory) {
        let interrupt_enable = memory.read(0xFFFF);
        let interrupt_flag = memory.read(0xFF0F);

        if self.interrupt_master_enable {
            if interrupt_enable & interrupt_flag & 0x1F == 0 {
            } else {
            }
        }
    }

    fn ld_r8_r8(&mut self, r8a: R8, r8b: R8, memory: &mut Memory) {
        let value = get_r8(r8b, &self.registers, memory);
        set_r8(r8a, value, &mut self.registers, memory);
    }

    fn halt(&mut self) {}

    fn add_a_r8(&mut self, r8: R8, memory: &Memory) {
        let a = self.registers.get_register8(Register8::A);
        let value = get_r8(r8, &self.registers, memory);
        let carry_value = self.registers.get_flag(Flag::Carry) as u8;

        let result = a.wrapping_add(value).wrapping_add(carry_value);

        self.registers.set_register8(Register8::A, result);

        let half_carry = (a & 0xF) + (value & 0xF) + carry_value > 0xF;

        let (sum, mut carry) = a.overflowing_add(value);

        if !carry {
            (_, carry) = sum.overflowing_add(carry_value);
        }

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Subtraction, false);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
        self.registers.set_flag(Flag::Carry, carry);
    }
}
