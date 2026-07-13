use crate::core::opcodes::{
    Cond, OperandType, R8, R16, R16mem, R16stk, decode_cond, decode_r8, decode_r16, decode_r16mem,
    decode_r16stk, get_cond, get_r8, get_r16, get_r16mem, get_r16stk, parse_operand, set_r8,
    set_r16, set_r16stk,
};
use crate::core::registers::{Flag, Register8, Register16};

use crate::core::{bus::Bus, registers::Registers};

pub enum Condition {
    NotZero,
    Zero,
    NotCarry,
    Carry,
}

#[derive(Debug, Clone, Copy)]
pub struct CycleInfo {
    pub cycle_count: u8,
    pub opcode_bytes: [u8; 3],
    pub opcode_address: u16,
    pub next_opcode_bytes: [u8; 3],
    pub next_opcode_address: u16,
    pub registers: Registers,
}

pub struct Cpu {
    registers: Registers,
    interrupt_master_enable: bool,
    interrupt_master_enable_pending: bool,
}

// Move immediate values to argument instead of getting inside function

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            interrupt_master_enable: false,
            interrupt_master_enable_pending: false,
        }
    }

    pub fn registers(&self) -> Registers {
        self.registers
    }

    fn fetch(&mut self, bus: &mut Bus) -> u8 {
        let pc = self.registers.get_register16(Register16::PC);
        self.registers.set_register16(Register16::PC, pc + 1);
        bus.read(pc)
    }

    fn fetch_16(&mut self, bus: &mut Bus) -> u16 {
        // Little-endian order
        (self.fetch(bus) as u16) | ((self.fetch(bus) as u16) << 8)
    }

    fn handle_interrupts(&mut self, bus: &mut Bus) {
        for i in 0..5 {
            let enabled = (bus.interrupts.enable >> i) & 1 == 1;
            let requested = (bus.interrupts.flag >> i) & 1 == 1;

            bus.interrupts.flag &= !(1 << i);
            self.interrupt_master_enable = false;

            if enabled && requested {
                // Push PC to stack
                let pc = self.registers.get_register16(Register16::PC);
                let mut sp = self.registers.get_register16(Register16::SP);

                sp = sp.wrapping_sub(1);
                bus.write(sp, (pc >> 8) as u8);

                sp = sp.wrapping_sub(1);
                bus.write(sp, (pc & 0xFF) as u8);

                self.registers.set_register16(Register16::SP, sp);

                // Jump to interrupt service routine
                let address = 0x40 + i * 0x8;
                self.registers.set_register16(Register16::PC, address);

                println!(
                    "{}",
                    match i {
                        0 => "vblank",
                        1 => "lcd",
                        2 => "timer",
                        3 => "serial",
                        4 => "joypad",
                        _ => unreachable!(),
                    }
                );
            }
        }
    }

    pub fn get_next_opcode(&self, bus: &mut Bus) -> ([u8; 3], u16) {
        let opcode_bytes = [
            bus.read(self.registers.pc),
            bus.read(self.registers.pc.wrapping_add(1)),
            bus.read(self.registers.pc.wrapping_add(2)),
        ];
        let opcode_address = self.registers.pc;

        (opcode_bytes, opcode_address)
    }

    // TODO: Rename from cycle as can be over multiple clock cycles
    pub fn cycle(&mut self, bus: &mut Bus) -> CycleInfo {
        let (opcode_bytes, opcode_address) = self.get_next_opcode(bus);

        // TODO: Would this be better at the end of function
        if self.interrupt_master_enable_pending {
            self.interrupt_master_enable = true;
            self.interrupt_master_enable_pending = false;
        }

        if self.interrupt_master_enable {
            self.handle_interrupts(bus);
        }

        let opcode = self.fetch(bus);

        let cycle_count = if opcode == 0b00000000 {
            self.nop()
        } else if opcode & 0b11001111 == 0b00000001 {
            let r16 = decode_r16(parse_operand(opcode, 4, OperandType::R16));
            self.ld_r16_imm16(r16, bus)
        } else if opcode & 0b11001111 == 0b00000010 {
            let r16mem = decode_r16mem(parse_operand(opcode, 4, OperandType::R16mem));
            self.ld_r16mem_a(r16mem, bus)
        } else if opcode & 0b11001111 == 0b00001010 {
            let r16mem = decode_r16mem(parse_operand(opcode, 4, OperandType::R16mem));
            self.ld_a_r16mem(r16mem, bus)
        } else if opcode == 0b00001000 {
            self.ld_imm16_sp(bus)
        } else if opcode & 0b11001111 == 0b00000011 {
            let r16 = decode_r16(parse_operand(opcode, 4, OperandType::R16));
            self.inc_r16(r16)
        } else if opcode & 0b11001111 == 0b00001011 {
            let r16 = decode_r16(parse_operand(opcode, 4, OperandType::R16));
            self.dec_r16(r16)
        } else if opcode & 0b11001111 == 0b00001001 {
            let r16 = decode_r16(parse_operand(opcode, 4, OperandType::R16));
            self.add_hl_r16(r16)
        } else if opcode & 0b11000111 == 0b00000100 {
            let r8 = decode_r8(parse_operand(opcode, 3, OperandType::R8));
            self.inc_r8(r8, bus)
        } else if opcode & 0b11000111 == 0b00000101 {
            let r8 = decode_r8(parse_operand(opcode, 3, OperandType::R8));
            self.dec_r8(r8, bus)
        } else if opcode & 0b11000111 == 0b00000110 {
            let r8 = decode_r8(parse_operand(opcode, 3, OperandType::R8));
            self.ld_r8_imm8(r8, bus)
        } else if opcode == 0b00000111 {
            self.rlca()
        } else if opcode == 0b00001111 {
            self.rrca()
        } else if opcode == 0b00010111 {
            self.rla()
        } else if opcode == 0b00011111 {
            self.rra()
        } else if opcode == 0b00100111 {
            self.daa()
        } else if opcode == 0b00101111 {
            self.cpl()
        } else if opcode == 0b00110111 {
            self.scf()
        } else if opcode == 0b00111111 {
            self.ccf()
        } else if opcode == 0b00011000 {
            self.jr_imm8(bus)
        } else if opcode & 0b11100111 == 0b00100000 {
            let cond = decode_cond(parse_operand(opcode, 3, OperandType::Cond));
            self.jr_cond_imm8(cond, bus)
        } else if opcode == 0b00010000 {
            todo!("stop")
        } else if opcode & 0b11000000 == 0b01000000 && opcode != 0b01110110 {
            let r8a = decode_r8(parse_operand(opcode, 3, OperandType::R8));
            let r8b = decode_r8(parse_operand(opcode, 0, OperandType::R8));
            self.ld_r8_r8(r8a, r8b, bus)
        } else if opcode == 0b01110110 {
            todo!("halt")
        } else if opcode & 0b11111000 == 0b10000000 {
            let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
            self.add_a_r8(r8, bus)
        } else if opcode & 0b11111000 == 0b10001000 {
            let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
            self.adc_a_r8(r8, bus)
        } else if opcode & 0b11111000 == 0b10010000 {
            let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
            self.sub_a_r8(r8, bus)
        } else if opcode & 0b11111000 == 0b10011000 {
            let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
            self.sbc_a_r8(r8, bus)
        } else if opcode & 0b11111000 == 0b10100000 {
            let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
            self.and_a_r8(r8, bus)
        } else if opcode & 0b11111000 == 0b10101000 {
            let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
            self.xor_a_r8(r8, bus)
        } else if opcode & 0b11111000 == 0b10110000 {
            let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
            self.or_a_r8(r8, bus)
        } else if opcode & 0b11111000 == 0b10111000 {
            let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
            self.cp_a_r8(r8, bus)
        } else if opcode == 0b11000110 {
            self.add_a_imm8(bus)
        } else if opcode == 0b11001110 {
            self.adc_a_imm8(bus)
        } else if opcode == 0b11010110 {
            self.sub_a_imm8(bus)
        } else if opcode == 0b11011110 {
            self.sbc_a_imm8(bus)
        } else if opcode == 0b11100110 {
            self.and_a_imm8(bus)
        } else if opcode == 0b11101110 {
            self.xor_a_imm8(bus)
        } else if opcode == 0b11110110 {
            self.or_a_imm8(bus)
        } else if opcode == 0b11111110 {
            self.cp_a_imm8(bus)
        } else if opcode & 0b11100111 == 0b11000000 {
            let cond = decode_cond(parse_operand(opcode, 3, OperandType::Cond));
            self.ret_cond(cond, bus)
        } else if opcode == 0b11001001 {
            self.ret(bus)
        } else if opcode == 0b11011001 {
            self.reti(bus)
        } else if opcode & 0b11100111 == 0b11000010 {
            let cond = decode_cond(parse_operand(opcode, 3, OperandType::Cond));
            self.jp_cond_imm16(cond, bus)
        } else if opcode == 0b11000011 {
            self.jp_imm16(bus)
        } else if opcode == 0b11101001 {
            self.jp_hl()
        } else if opcode & 0b11100111 == 0b11000100 {
            let cond = decode_cond(parse_operand(opcode, 3, OperandType::Cond));
            self.call_cond_imm16(cond, bus)
        } else if opcode == 0b11001101 {
            self.call_imm16(bus)
        } else if opcode & 0b11000111 == 0b11000111 {
            let tgt3 = parse_operand(opcode, 3, OperandType::Tgt3);
            self.rst_tgt3(tgt3, bus)
        } else if opcode & 0b11001111 == 0b11000001 {
            let r16stk = decode_r16stk(parse_operand(opcode, 3, OperandType::R16stk));
            self.pop_r16stk(r16stk, bus)
        } else if opcode & 0b11001111 == 0b11000101 {
            let r16stk = decode_r16stk(parse_operand(opcode, 3, OperandType::R16stk));
            self.push_r16stk(r16stk, bus)
        } else if opcode == 0b11001011 {
            let opcode = self.fetch(bus);

            if opcode & 0b11111000 == 0b00000000 {
                let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
                self.rlc_r8(r8, bus)
            } else if opcode & 0b11111000 == 0b00001000 {
                let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
                self.rrc_r8(r8, bus)
            } else if opcode & 0b11111000 == 0b00010000 {
                let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
                self.rl_r8(r8, bus)
            } else if opcode & 0b11111000 == 0b00011000 {
                let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
                self.rr_r8(r8, bus)
            } else if opcode & 0b11111000 == 0b00100000 {
                let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
                self.sla_r8(r8, bus)
            } else if opcode & 0b11111000 == 0b00101000 {
                let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
                self.sra_r8(r8, bus)
            } else if opcode & 0b11111000 == 0b00110000 {
                let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
                self.swap_r8(r8, bus)
            } else if opcode & 0b11111000 == 0b00111000 {
                let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
                self.srl_r8(r8, bus)
            } else if opcode & 0b11000000 == 0b01000000 {
                let b3 = parse_operand(opcode, 3, OperandType::B3);
                let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
                self.bit_b3_r8(b3, r8, bus)
            } else if opcode & 0b11000000 == 0b10000000 {
                let b3 = parse_operand(opcode, 3, OperandType::B3);
                let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
                self.res_b3_r8(b3, r8, bus)
            } else if opcode & 0b11000000 == 0b11000000 {
                let b3 = parse_operand(opcode, 3, OperandType::B3);
                let r8 = decode_r8(parse_operand(opcode, 0, OperandType::R8));
                self.set_b3_r8(b3, r8, bus)
            } else {
                1 // TODO: Fix this
            }
        } else if opcode == 0b11100010 {
            self.ldh_c_a(bus)
        } else if opcode == 0b11100000 {
            self.ldh_imm8_a(bus)
        } else if opcode == 0b11101010 {
            self.ld_imm16_a(bus)
        } else if opcode == 0b11110010 {
            self.ldh_a_c(bus)
        } else if opcode == 0b11110000 {
            self.ldh_a_imm8(bus)
        } else if opcode == 0b11111010 {
            self.ld_a_imm16(bus)
        } else if opcode == 0b11101000 {
            self.add_sp_imm8(bus)
        } else if opcode == 0b11111000 {
            self.ld_hl_sp_imm8(bus)
        } else if opcode == 0b11111001 {
            self.ld_sp_hl()
        } else if opcode == 0b11110011 {
            self.di()
        } else if opcode == 0b11111011 {
            self.ei()
        } else {
            1 // TODO: Fix this
        };

        let (next_opcode_bytes, next_opcode_address) = self.get_next_opcode(bus);

        CycleInfo {
            cycle_count,
            opcode_bytes,
            opcode_address,
            next_opcode_bytes,
            next_opcode_address,
            registers: self.registers,
        }
    }

    fn nop(&self) -> u8 {
        1
    }

    fn ld_r16_imm16(&mut self, r16: R16, bus: &mut Bus) -> u8 {
        let value = self.fetch_16(bus);
        set_r16(r16, value, &mut self.registers);
        3
    }

    fn ld_r16mem_a(&mut self, r16mem: R16mem, bus: &mut Bus) -> u8 {
        let address = get_r16mem(r16mem, &self.registers);
        bus.write(address, self.registers.a);

        // TODO: Move this to a function
        match r16mem {
            R16mem::HLI => self.registers.set_register16(
                Register16::HL,
                self.registers
                    .get_register16(Register16::HL)
                    .wrapping_add(1),
            ),
            R16mem::HLD => self.registers.set_register16(
                Register16::HL,
                self.registers
                    .get_register16(Register16::HL)
                    .wrapping_sub(1),
            ),
            _ => (),
        }

        2
    }

    fn ld_a_r16mem(&mut self, r16mem: R16mem, bus: &mut Bus) -> u8 {
        let address = get_r16mem(r16mem, &self.registers);
        self.registers.a = bus.read(address);

        match r16mem {
            R16mem::HLI => self.registers.set_register16(
                Register16::HL,
                self.registers
                    .get_register16(Register16::HL)
                    .wrapping_add(1),
            ),
            R16mem::HLD => self.registers.set_register16(
                Register16::HL,
                self.registers
                    .get_register16(Register16::HL)
                    .wrapping_sub(1),
            ),
            _ => (),
        }

        2
    }

    fn ld_imm16_sp(&mut self, bus: &mut Bus) -> u8 {
        let address = self.fetch_16(bus);
        let value = self.registers.sp;
        bus.write(address, (value & 0xFF) as u8);
        bus.write(address + 1, (value >> 8) as u8);
        5
    }

    fn inc_r16(&mut self, r16: R16) -> u8 {
        let value = get_r16(r16, &self.registers);
        let result = value.wrapping_add(1);
        set_r16(r16, result, &mut self.registers);
        2
    }

    fn dec_r16(&mut self, r16: R16) -> u8 {
        let value = get_r16(r16, &self.registers);
        let result = value.wrapping_sub(1);
        set_r16(r16, result, &mut self.registers);
        2
    }

    fn add_hl_r16(&mut self, r16: R16) -> u8 {
        let hl = self.registers.get_register16(Register16::HL);
        let value = get_r16(r16, &self.registers);

        let (result, carry) = hl.overflowing_add(value);
        self.registers.set_register16(Register16::HL, result);

        let half_carry = (hl & 0xFFF) + (value & 0xFFF) > 0xFFF;

        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
        self.registers.set_flag(Flag::Carry, carry);

        2
    }

    fn inc_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let value = get_r8(r8, &mut self.registers, bus);
        let result = value.wrapping_add(1);
        set_r8(r8, result, &mut self.registers, bus);

        let half_carry = (value & 0xF) + 1 > 0xF;

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, half_carry);

        1
    }

    fn dec_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let value = get_r8(r8, &self.registers, bus);
        let result = value.wrapping_sub(1);
        set_r8(r8, result, &mut self.registers, bus);

        let half_carry = value & 0xF == 0;

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, true);
        self.registers.set_flag(Flag::HalfCarry, half_carry);

        1
    }

    fn ld_r8_imm8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let value = self.fetch(bus);
        set_r8(r8, value, &mut self.registers, bus);
        2
    }

    fn rlca(&mut self) -> u8 {
        let value = self.registers.get_register8(Register8::A);
        self.registers
            .set_register8(Register8::A, value.rotate_left(1));

        self.registers.set_flag(Flag::Zero, false);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, value >> 7 == 1);

        1
    }

    fn rrca(&mut self) -> u8 {
        let value = self.registers.get_register8(Register8::A);
        self.registers
            .set_register8(Register8::A, value.rotate_right(1));

        self.registers.set_flag(Flag::Zero, false);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, value & 1 == 1);

        1
    }

    fn rla(&mut self) -> u8 {
        let value = self.registers.get_register8(Register8::A);
        let carry = (value >> 7) == 1;
        let mut result = value << 1;

        if self.registers.get_flag(Flag::Carry) {
            result |= 1;
        }

        self.registers.set_register8(Register8::A, result);

        self.registers.set_flag(Flag::Zero, false);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, carry);

        1
    }

    fn rra(&mut self) -> u8 {
        let value = self.registers.get_register8(Register8::A);
        let carry = value & 1 == 1;
        let mut result = value >> 1;

        if self.registers.get_flag(Flag::Carry) {
            result |= 1 << 7;
        }

        self.registers.set_register8(Register8::A, result);

        self.registers.set_flag(Flag::Zero, false);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, carry);

        1
    }

    fn daa(&mut self) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let half_carry = self.registers.get_flag(Flag::HalfCarry);
        let carry = self.registers.get_flag(Flag::Carry);

        if self.registers.get_flag(Flag::Negative) {
            let mut adjustment: u8 = 0;

            if half_carry {
                adjustment += 0x6;
            }

            if carry {
                adjustment += 0x60;
            }

            self.registers
                .set_register8(Register8::A, a.wrapping_sub(adjustment));
        } else {
            let mut adjustment: u8 = 0;

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

        self.registers
            .set_flag(Flag::Zero, self.registers.get_register8(Register8::A) == 0);
        self.registers.set_flag(Flag::HalfCarry, false);

        1
    }

    fn cpl(&mut self) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        self.registers.set_register8(Register8::A, !a);

        self.registers.set_flag(Flag::Negative, true);
        self.registers.set_flag(Flag::HalfCarry, true);

        1
    }

    fn scf(&mut self) -> u8 {
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, true);

        1
    }

    fn ccf(&mut self) -> u8 {
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);

        let carry = self.registers.get_flag(Flag::Carry);
        self.registers.set_flag(Flag::Carry, !carry);

        1
    }

    fn jr_imm8(&mut self, bus: &mut Bus) -> u8 {
        let offset = self.fetch(bus) as i8;
        let pc = self.registers.get_register16(Register16::PC);

        self.registers
            .set_register16(Register16::PC, pc.wrapping_add_signed(offset as i16));

        3
    }

    fn jr_cond_imm8(&mut self, cond: Cond, bus: &mut Bus) -> u8 {
        if get_cond(cond, &self.registers) {
            self.jr_imm8(bus);
            3
        } else {
            let pc = self.registers.get_register16(Register16::PC);
            self.registers.set_register16(Register16::PC, pc + 1);
            2
        }
    }

    // TODO: Implement this
    fn stop(&mut self, bus: &mut Bus) -> u8 {
        let interrupt_enable = bus.read(0xFFFF);
        let interrupt_flag = bus.read(0xFF0F);

        if self.interrupt_master_enable {
            if interrupt_enable & interrupt_flag & 0x1F == 0 {
            } else {
            }
        }

        1 // TODO: This needs changing
    }

    fn ld_r8_r8(&mut self, r8a: R8, r8b: R8, bus: &mut Bus) -> u8 {
        let value = get_r8(r8b, &self.registers, bus);
        set_r8(r8a, value, &mut self.registers, bus);
        1
    }

    // TODO: Implement this
    fn halt(&mut self) -> u8 {
        1 // TODO: This needs fixing
    }

    fn add_a_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = get_r8(r8, &self.registers, bus);

        let (result, carry) = a.overflowing_add(value);

        self.registers.set_register8(Register8::A, result);

        let half_carry = (a & 0xF) + (value & 0xF) > 0xF;

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
        self.registers.set_flag(Flag::Carry, carry);

        1
    }

    fn adc_a_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = get_r8(r8, &self.registers, bus);
        let carry_value = self.registers.get_flag(Flag::Carry) as u8;

        let result = a.wrapping_add(value).wrapping_add(carry_value);

        self.registers.set_register8(Register8::A, result);

        let half_carry = (a & 0xF) + (value & 0xF) + carry_value > 0xF;
        let carry = ((a as u16) & 0xFF) + ((value as u16) & 0xFF) + (carry_value as u16) > 0xFF;

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
        self.registers.set_flag(Flag::Carry, carry);

        1
    }

    fn sub_a_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = get_r8(r8, &self.registers, bus);

        let result = a.wrapping_sub(value);

        self.registers.set_register8(Register8::A, result);

        let half_carry = (a & 0xF) < (value & 0xF);
        let carry = a < value;

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, true);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
        self.registers.set_flag(Flag::Carry, carry);

        1
    }

    fn sbc_a_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = get_r8(r8, &self.registers, bus);
        let carry_value = self.registers.get_flag(Flag::Carry) as u8;

        let result = a.wrapping_sub(value).wrapping_sub(carry_value);

        self.registers.set_register8(Register8::A, result);

        let half_carry = (a & 0xF) < (value & 0xF) + carry_value;
        let carry = (a as u16) < (value as u16) + (carry_value as u16);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, true);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
        self.registers.set_flag(Flag::Carry, carry);

        1
    }

    fn and_a_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = get_r8(r8, &self.registers, bus);
        let result = a & value;

        self.registers.set_register8(Register8::A, result);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, true);
        self.registers.set_flag(Flag::Carry, false);

        1
    }

    fn xor_a_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = get_r8(r8, &self.registers, bus);
        let result = a ^ value;

        self.registers.set_register8(Register8::A, result);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, false);

        1
    }

    fn or_a_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = get_r8(r8, &self.registers, bus);
        let result = a | value;

        self.registers.set_register8(Register8::A, result);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, false);

        1
    }

    fn cp_a_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = get_r8(r8, &self.registers, bus);

        let result = a.wrapping_sub(value);

        let half_carry = (a & 0xF) < (value & 0xF);
        let carry = a < value;

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, true);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
        self.registers.set_flag(Flag::Carry, carry);

        1
    }

    fn add_a_imm8(&mut self, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = self.fetch(bus);

        let (result, carry) = a.overflowing_add(value);

        self.registers.set_register8(Register8::A, result);

        let half_carry = (a & 0xF) + (value & 0xF) > 0xF;

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
        self.registers.set_flag(Flag::Carry, carry);

        2
    }

    fn adc_a_imm8(&mut self, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = self.fetch(bus);
        let carry_value = self.registers.get_flag(Flag::Carry) as u8;

        let result = a.wrapping_add(value).wrapping_add(carry_value);

        self.registers.set_register8(Register8::A, result);

        let half_carry = (a & 0xF) + (value & 0xF) + carry_value > 0xF;
        let carry = ((a as u16) & 0xFF) + ((value as u16) & 0xFF) + (carry_value as u16) > 0xFF;

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
        self.registers.set_flag(Flag::Carry, carry);

        2
    }

    fn sub_a_imm8(&mut self, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = self.fetch(bus);

        let result = a.wrapping_sub(value);

        self.registers.set_register8(Register8::A, result);

        let half_carry = (a & 0xF) < (value & 0xF);
        let carry = a < value;

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, true);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
        self.registers.set_flag(Flag::Carry, carry);

        2
    }

    fn sbc_a_imm8(&mut self, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = self.fetch(bus);
        let carry_value = self.registers.get_flag(Flag::Carry) as u8;

        let result = a.wrapping_sub(value).wrapping_sub(carry_value);

        self.registers.set_register8(Register8::A, result);

        let half_carry = (a & 0xF) < (value & 0xF) + carry_value;
        let carry = (a as u16) < (value as u16) + (carry_value as u16);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, true);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
        self.registers.set_flag(Flag::Carry, carry);

        2
    }

    fn and_a_imm8(&mut self, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = self.fetch(bus);
        let result = a & value;

        self.registers.set_register8(Register8::A, result);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, true);
        self.registers.set_flag(Flag::Carry, false);

        2
    }

    fn xor_a_imm8(&mut self, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = self.fetch(bus);
        let result = a ^ value;

        self.registers.set_register8(Register8::A, result);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, false);

        2
    }

    fn or_a_imm8(&mut self, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = self.fetch(bus);
        let result = a | value;

        self.registers.set_register8(Register8::A, result);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, false);

        2
    }

    fn cp_a_imm8(&mut self, bus: &mut Bus) -> u8 {
        let a = self.registers.get_register8(Register8::A);
        let value = self.fetch(bus);

        let result = a.wrapping_sub(value);

        let half_carry = (a & 0xF) < (value & 0xF);
        let carry = a < value;

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, true);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
        self.registers.set_flag(Flag::Carry, carry);

        2
    }

    fn ret_cond(&mut self, cond: Cond, bus: &mut Bus) -> u8 {
        if get_cond(cond, &self.registers) {
            self.ret(bus);
            5
        } else {
            let pc = self.registers.get_register16(Register16::PC);
            self.registers.set_register16(Register16::PC, pc + 2);
            2
        }
    }

    fn ret(&mut self, bus: &mut Bus) -> u8 {
        let mut sp = self.registers.get_register16(Register16::SP);

        let mut value = bus.read(sp) as u16;
        sp = sp.wrapping_add(1);

        value += (bus.read(sp) as u16) << 8;
        sp = sp.wrapping_add(1);

        self.registers.set_register16(Register16::SP, sp);
        self.registers.set_register16(Register16::PC, value);

        4
    }

    fn reti(&mut self, bus: &mut Bus) -> u8 {
        self.ei();
        self.ret(bus);
        4
    }

    fn jp_cond_imm16(&mut self, cond: Cond, bus: &mut Bus) -> u8 {
        if get_cond(cond, &self.registers) {
            self.jp_imm16(bus);
            4
        } else {
            let pc = self.registers.get_register16(Register16::PC);
            self.registers.set_register16(Register16::PC, pc + 2);
            3
        }
    }

    fn jp_imm16(&mut self, bus: &mut Bus) -> u8 {
        let address = self.fetch_16(bus);
        self.registers.set_register16(Register16::PC, address);
        4
    }

    fn jp_hl(&mut self) -> u8 {
        let hl = self.registers.get_register16(Register16::HL);
        self.registers.set_register16(Register16::PC, hl);
        1
    }

    fn call_cond_imm16(&mut self, cond: Cond, bus: &mut Bus) -> u8 {
        if get_cond(cond, &self.registers) {
            self.call_imm16(bus);
            6
        } else {
            let pc = self.registers.get_register16(Register16::PC);
            self.registers.set_register16(Register16::PC, pc + 2);
            3
        }
    }

    fn call_imm16(&mut self, bus: &mut Bus) -> u8 {
        let address = self.fetch_16(bus);

        let mut sp = self.registers.get_register16(Register16::SP);
        let pc = self.registers.get_register16(Register16::PC);

        sp = sp.wrapping_sub(1);
        bus.write(sp, (pc >> 8) as u8);

        sp = sp.wrapping_sub(1);
        bus.write(sp, (pc & 0xFF) as u8);

        self.registers.set_register16(Register16::SP, sp);
        self.registers.set_register16(Register16::PC, address);

        6
    }

    fn rst_tgt3(&mut self, tgt3: u8, bus: &mut Bus) -> u8 {
        let address = 8 * (tgt3 as u16);

        let mut sp = self.registers.get_register16(Register16::SP);
        let pc = self.registers.get_register16(Register16::PC);

        sp = sp.wrapping_sub(1);
        bus.write(sp, (pc >> 8) as u8);

        sp = sp.wrapping_sub(1);
        bus.write(sp, (pc & 0xFF) as u8);

        self.registers.set_register16(Register16::SP, sp);
        self.registers.set_register16(Register16::PC, address);

        4
    }

    fn pop_r16stk(&mut self, r16stk: R16stk, bus: &mut Bus) -> u8 {
        let mut sp = self.registers.get_register16(Register16::SP);

        let mut value = bus.read(sp) as u16;
        sp = sp.wrapping_add(1);

        value += (bus.read(sp) as u16) << 8;
        sp = sp.wrapping_add(1);

        self.registers.set_register16(Register16::SP, sp);
        set_r16stk(r16stk, value, &mut self.registers);

        3
    }

    fn push_r16stk(&mut self, r16stk: R16stk, bus: &mut Bus) -> u8 {
        let value = get_r16stk(r16stk, &self.registers);
        let mut sp = self.registers.get_register16(Register16::SP);

        sp = sp.wrapping_sub(1);
        bus.write(sp, (value >> 8) as u8);

        sp = sp.wrapping_sub(1);
        bus.write(sp, (value & 0xFF) as u8);

        self.registers.set_register16(Register16::SP, sp);

        4
    }

    fn rlc_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let value = get_r8(r8, &self.registers, bus);
        let result = value.rotate_left(1);
        set_r8(r8, result, &mut self.registers, bus);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, value >> 7 == 1);

        2
    }

    fn rrc_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let value = get_r8(r8, &self.registers, bus);
        let result = value.rotate_right(1);
        set_r8(r8, result, &mut self.registers, bus);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, value & 1 == 1);

        2
    }

    fn rl_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let value = get_r8(r8, &self.registers, bus);
        let carry = (value >> 7) == 1;
        let mut result = value << 1;

        if self.registers.get_flag(Flag::Carry) {
            result |= 1;
        }

        set_r8(r8, result, &mut self.registers, bus);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, carry);

        2
    }

    fn rr_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let value = get_r8(r8, &self.registers, bus);
        let carry = value & 1 == 1;
        let mut result = value >> 1;

        if self.registers.get_flag(Flag::Carry) {
            result |= 1 << 7;
        }

        set_r8(r8, result, &mut self.registers, bus);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, carry);

        2
    }

    fn sla_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let value = get_r8(r8, &self.registers, bus);
        let result = value << 1;
        let carry = value >> 7 == 1;

        set_r8(r8, result, &mut self.registers, bus);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, carry);

        2
    }

    fn sra_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let value = get_r8(r8, &self.registers, bus);
        let mut result = value >> 1;

        if value >> 7 == 1 {
            result |= 0b10000000
        }

        set_r8(r8, result, &mut self.registers, bus);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, value & 1 == 1);

        2
    }

    fn swap_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let value = get_r8(r8, &self.registers, bus);
        let result = (value << 4) | (value >> 4);
        set_r8(r8, result, &mut self.registers, bus);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, false);

        2
    }

    fn srl_r8(&mut self, r8: R8, bus: &mut Bus) -> u8 {
        let value = get_r8(r8, &self.registers, bus);
        let result = value >> 1;
        set_r8(r8, value, &mut self.registers, bus);

        self.registers.set_flag(Flag::Zero, result == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, false);
        self.registers.set_flag(Flag::Carry, value & 1 == 1);

        2
    }

    fn bit_b3_r8(&mut self, b3: u8, r8: R8, bus: &mut Bus) -> u8 {
        let value = get_r8(r8, &self.registers, bus);
        let bit = (value >> b3) & 1;

        self.registers.set_flag(Flag::Zero, bit == 0);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, true);

        2
    }

    fn res_b3_r8(&mut self, b3: u8, r8: R8, bus: &mut Bus) -> u8 {
        let value = get_r8(r8, &self.registers, bus);
        let result = value & !(1 << b3);
        set_r8(r8, result, &mut self.registers, bus);
        2
    }

    fn set_b3_r8(&mut self, b3: u8, r8: R8, bus: &mut Bus) -> u8 {
        let value = get_r8(r8, &self.registers, bus);
        let result = value | (1 << b3);
        set_r8(r8, result, &mut self.registers, bus);
        2
    }

    fn ldh_c_a(&mut self, bus: &mut Bus) -> u8 {
        let address = 0xFF00 + (self.registers.get_register8(Register8::C) as u16);
        let a = self.registers.get_register8(Register8::A);
        bus.write(address, a);
        2
    }

    fn ldh_imm8_a(&mut self, bus: &mut Bus) -> u8 {
        let address = 0xFF00 + (self.fetch(bus) as u16);
        let a = self.registers.get_register8(Register8::A);
        bus.write(address, a);
        3
    }

    fn ld_imm16_a(&mut self, bus: &mut Bus) -> u8 {
        let imm16 = self.fetch_16(bus);
        let a = self.registers.get_register8(Register8::A);
        bus.write(imm16, a);
        4
    }

    fn ldh_a_c(&mut self, bus: &mut Bus) -> u8 {
        let address = 0xFF00 + (self.registers.get_register8(Register8::C) as u16);
        let value = bus.read(address);
        self.registers.set_register8(Register8::A, value);
        2
    }

    fn ldh_a_imm8(&mut self, bus: &mut Bus) -> u8 {
        let address = 0xFF00 + self.fetch(bus) as u16;
        let value = bus.read(address);
        self.registers.set_register8(Register8::A, value);
        3
    }

    fn ld_a_imm16(&mut self, bus: &mut Bus) -> u8 {
        let imm16 = self.fetch_16(bus);
        let value = bus.read(imm16);
        self.registers.set_register8(Register8::A, value);
        4
    }

    fn add_sp_imm8(&mut self, bus: &mut Bus) -> u8 {
        let sp = self.registers.get_register16(Register16::SP);
        let value = self.fetch(bus) as i8;
        let result = sp.wrapping_add_signed(value as i16);

        self.registers.set_register16(Register16::SP, result);

        let half_carry = ((sp as u8) & 0xF) + ((value as u8) & 0xF) > 0xF;
        let carry = sp + (value as u8 as u16) > 0xFF;

        self.registers.set_flag(Flag::Zero, false);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
        self.registers.set_flag(Flag::Carry, carry);

        4
    }

    fn ld_hl_sp_imm8(&mut self, bus: &mut Bus) -> u8 {
        let value = self.fetch(bus) as i8;
        let sp = self.registers.get_register16(Register16::SP);

        self.registers
            .set_register16(Register16::HL, sp.wrapping_add_signed(value as i16));

        let half_carry = (sp & 0xF) + ((value as u8 as u16) & 0xF) > 0xF;
        let carry = (sp & 0xFF) + ((value as u8 as u16) & 0xFF) > 0xFF;

        self.registers.set_flag(Flag::Zero, false);
        self.registers.set_flag(Flag::Negative, false);
        self.registers.set_flag(Flag::HalfCarry, half_carry);
        self.registers.set_flag(Flag::Carry, carry);

        3
    }

    fn ld_sp_hl(&mut self) -> u8 {
        let hl = self.registers.get_register16(Register16::HL);
        self.registers.set_register16(Register16::SP, hl);
        2
    }

    fn di(&mut self) -> u8 {
        self.interrupt_master_enable = false;
        1
    }

    fn ei(&mut self) -> u8 {
        self.interrupt_master_enable_pending = true;
        1
    }
}
