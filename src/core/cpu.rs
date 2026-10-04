use crate::core::opcodes::{
    Cond, OperandType, R8, R16, R16mem, R16stk, decode_cond, decode_r8, decode_r16, decode_r16mem,
    decode_r16stk, get_cond, get_r8, get_r16, get_r16mem, get_r16stk, parse_operand, set_r8,
    set_r16, set_r16mem, set_r16stk,
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

pub enum Instruction {
    Unimplemented,
    Nop,
    LdR16Imm16(R16),
    LdR16memA(R16mem),
    LdAR16mem(R16mem),
    LdImm16Sp,
    IncR16(R16),
    DecR16(R16),
    AddHlR16(R16),
    IncR8(R8),
    DecR8(R8),
    LdR8Imm8(R8),
    Rlca,
    Rrca,
    Rla,
    Rra,
    Daa,
    Cpl,
    Scf,
    Ccf,
    JrImm8,
    JrCondImm8(Cond),
    Stop,
    LdR8R8(R8, R8),
    Halt,
    AddAR8(R8),
    AdcAR8(R8),
    SubAR8(R8),
    SbcAR8(R8),
    AndAR8(R8),
    XorAR8(R8),
    OrAR8(R8),
    CpAR8(R8),
    AddAImm8,
    AdcAImm8,
    SubAImm8,
    SbcAImm8,
    AndAImm8,
    XorAImm8,
    OrAImm8,
    CpAImm8,
    RetCond(Cond),
    Ret,
    Reti,
    JpCondImm16(Cond),
    JpImm16,
    JpHl,
    CallCondImm16(Cond),
    CallImm16,
    RstTgt3(u8),
    PopR16stk(R16stk),
    PushR16stk(R16stk),
    RlcR8(R8),
    RrcR8(R8),
    RlR8(R8),
    RrR8(R8),
    SlaR8(R8),
    SraR8(R8),
    SwapR8(R8),
    SrlR8(R8),
    BitB3R8(u8, R8),
    ResB3R8(u8, R8),
    SetB3R8(u8, R8),
    LdhCA,
    LdhImm8A,
    LdImm16A,
    LdhAC,
    LdhAImm8,
    LdAImm16,
    AddSpImm8,
    LdHlSpImm8,
    LdSpHl,
    Di,
    Ei,
}

pub enum CpuState {
    Fetching,
    Executing(Instruction, u8),
    Halted,
}

pub struct Cpu {
    halted: bool,
    registers: Registers,
    cycle_counter: u8,
    cb_prefix: bool,
    sign: bool, // Maintains sign between cycles
    interrupt_master_enable: bool,
    interrupt_master_enable_pending: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            halted: false,
            registers: Registers::new(),
            cycle_counter: 0,
            cb_prefix: true,
            sign: false,
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

    fn handle_interrupts(&mut self, bus: &mut Bus) -> u8 {
        if ((bus.graphics.stat() >> 3) & 1 == 1 && (bus.graphics.stat() & 0b00000011) == 0b00000000)
            || ((bus.graphics.stat() >> 4) & 1 == 1
                && (bus.graphics.stat() & 0b00000011) == 0b00000001)
            || ((bus.graphics.stat() >> 5) & 1 == 1
                && (bus.graphics.stat() & 0b00000011) == 0b00000010)
            || ((bus.graphics.stat() >> 6) & 1 == 1 && (bus.graphics.stat() >> 2) & 1 == 1)
        {
            bus.interrupts.flag |= 1 << 1;
        }

        let mut cycle_count = 0;

        for i in 0..=4 {
            let enabled = (bus.interrupts.enable >> i) & 1 == 1;
            let requested = (bus.interrupts.flag >> i) & 1 == 1;

            if enabled && requested {
                bus.interrupts.flag &= !(1 << i);
                self.interrupt_master_enable = false;

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

                cycle_count += 5;

                // println!(
                //     "{}",
                //     match i {
                //         0 => "vblank",
                //         1 => "lcd",
                //         2 => "timer",
                //         3 => "serial",
                //         4 => "joypad",
                //         _ => unreachable!(),
                //     }
                // );
            }
        }

        cycle_count
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

    pub fn decode(&mut self, bus: &mut Bus) {
        self.registers.ir = bus.read(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);

        let ir = self.registers.ir;

        if !self.cb_prefix {
            if ir == 0b00000000 {
                self.nop();
            } else if ir & 0b11001111 == 0b00000001 {
                let r16 = decode_r16(parse_operand(ir, 4, OperandType::R16));
                self.ld_r16_imm16(r16, bus);
            } else if ir & 0b11001111 == 0b00000010 {
                let r16mem = decode_r16mem(parse_operand(ir, 4, OperandType::R16mem));
                self.ld_r16mem_a(r16mem, bus);
            } else if ir & 0b11001111 == 0b00001010 {
                let r16mem = decode_r16mem(parse_operand(ir, 4, OperandType::R16mem));
                self.ld_a_r16mem(r16mem, bus);
            } else if ir == 0b00001000 {
                self.ld_imm16_sp(bus);
            } else if ir & 0b11001111 == 0b00000011 {
                let r16 = decode_r16(parse_operand(ir, 4, OperandType::R16));
                self.inc_r16(r16);
            } else if ir & 0b11001111 == 0b00001011 {
                let r16 = decode_r16(parse_operand(ir, 4, OperandType::R16));
                self.dec_r16(r16);
            } else if ir & 0b11001111 == 0b00001001 {
                let r16 = decode_r16(parse_operand(ir, 4, OperandType::R16));
                self.add_hl_r16(r16);
            } else if ir & 0b11000111 == 0b00000100 {
                let r8 = decode_r8(parse_operand(ir, 3, OperandType::R8));
                self.inc_r8(r8, bus);
            } else if ir & 0b11000111 == 0b00000101 {
                let r8 = decode_r8(parse_operand(ir, 3, OperandType::R8));
                self.dec_r8(r8, bus);
            } else if ir & 0b11000111 == 0b00000110 {
                let r8 = decode_r8(parse_operand(ir, 3, OperandType::R8));
                self.ld_r8_imm8(r8, bus);
            } else if ir == 0b00000111 {
                self.rlca();
            } else if ir == 0b00001111 {
                self.rrca();
            } else if ir == 0b00010111 {
                self.rla();
            } else if ir == 0b00011111 {
                self.rra();
            } else if ir == 0b00100111 {
                self.daa();
            } else if ir == 0b00101111 {
                self.cpl();
            } else if ir == 0b00110111 {
                self.scf();
            } else if ir == 0b00111111 {
                self.ccf();
            } else if ir == 0b00011000 {
                self.jr_imm8(bus);
            } else if ir & 0b11100111 == 0b00100000 {
                let cond = decode_cond(parse_operand(ir, 3, OperandType::Cond));
                self.jr_cond_imm8(cond, bus);
            } else if ir == 0b00010000 {
                self.stop(bus);
            } else if ir & 0b11000000 == 0b01000000 && ir != 0b01110110 {
                let r8a = decode_r8(parse_operand(ir, 3, OperandType::R8));
                let r8b = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.ld_r8_r8(r8a, r8b, bus);
            } else if ir == 0b01110110 {
                self.halt();
            } else if ir & 0b11111000 == 0b10000000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.add_a_r8(r8, bus);
            } else if ir & 0b11111000 == 0b10001000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.adc_a_r8(r8, bus);
            } else if ir & 0b11111000 == 0b10010000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.sub_a_r8(r8, bus);
            } else if ir & 0b11111000 == 0b10011000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.sbc_a_r8(r8, bus);
            } else if ir & 0b11111000 == 0b10100000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.and_a_r8(r8, bus);
            } else if ir & 0b11111000 == 0b10101000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.xor_a_r8(r8, bus);
            } else if ir & 0b11111000 == 0b10110000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.or_a_r8(r8, bus);
            } else if ir & 0b11111000 == 0b10111000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.cp_a_r8(r8, bus);
            } else if ir == 0b11000110 {
                self.add_a_imm8(bus);
            } else if ir == 0b11001110 {
                self.adc_a_imm8(bus);
            } else if ir == 0b11010110 {
                self.sub_a_imm8(bus);
            } else if ir == 0b11011110 {
                self.sbc_a_imm8(bus);
            } else if ir == 0b11100110 {
                self.and_a_imm8(bus);
            } else if ir == 0b11101110 {
                self.xor_a_imm8(bus);
            } else if ir == 0b11110110 {
                self.or_a_imm8(bus);
            } else if ir == 0b11111110 {
                self.cp_a_imm8(bus);
            } else if ir & 0b11100111 == 0b11000000 {
                let cond = decode_cond(parse_operand(ir, 3, OperandType::Cond));
                self.ret_cond(cond, bus);
            } else if ir == 0b11001001 {
                self.ret(bus);
            } else if ir == 0b11011001 {
                self.reti(bus);
            } else if ir & 0b11100111 == 0b11000010 {
                let cond = decode_cond(parse_operand(ir, 3, OperandType::Cond));
                self.jp_cond_imm16(cond, bus);
            } else if ir == 0b11000011 {
                self.jp_imm16(bus);
            } else if ir == 0b11101001 {
                self.jp_hl();
            } else if ir & 0b11100111 == 0b11000100 {
                let cond = decode_cond(parse_operand(ir, 3, OperandType::Cond));
                self.call_cond_imm16(cond, bus);
            } else if ir == 0b11001101 {
                self.call_imm16(bus);
            } else if ir & 0b11000111 == 0b11000111 {
                let tgt3 = parse_operand(ir, 3, OperandType::Tgt3);
                self.rst_tgt3(tgt3, bus);
            } else if ir & 0b11001111 == 0b11000001 {
                let r16stk = decode_r16stk(parse_operand(ir, 4, OperandType::R16stk));
                self.pop_r16stk(r16stk, bus);
            } else if ir & 0b11001111 == 0b11000101 {
                let r16stk = decode_r16stk(parse_operand(ir, 4, OperandType::R16stk));
                self.push_r16stk(r16stk, bus);
            } else if ir == 0b11001011 {
                self.cb_prefix = true;
            } else if ir == 0b11100010 {
                self.ldh_c_a(bus);
            } else if ir == 0b11100000 {
                self.ldh_imm8_a(bus);
            } else if ir == 0b11101010 {
                self.ld_imm16_a(bus);
            } else if ir == 0b11110010 {
                self.ldh_a_c(bus);
            } else if ir == 0b11110000 {
                self.ldh_a_imm8(bus);
            } else if ir == 0b11111010 {
                self.ld_a_imm16(bus);
            } else if ir == 0b11101000 {
                self.add_sp_imm8(bus);
            } else if ir == 0b11111000 {
                self.ld_hl_sp_imm8(bus);
            } else if ir == 0b11111001 {
                self.ld_sp_hl();
            } else if ir == 0b11110011 {
                self.di();
            } else if ir == 0b11111011 {
                self.ei();
            }
        } else {
            if ir & 0b11111000 == 0b00000000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.rlc_r8(r8, bus);
            } else if ir & 0b11111000 == 0b00001000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.rrc_r8(r8, bus);
            } else if ir & 0b11111000 == 0b00010000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.rl_r8(r8, bus);
            } else if ir & 0b11111000 == 0b00011000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.rr_r8(r8, bus);
            } else if ir & 0b11111000 == 0b00100000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.sla_r8(r8, bus);
            } else if ir & 0b11111000 == 0b00101000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.sra_r8(r8, bus);
            } else if ir & 0b11111000 == 0b00110000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.swap_r8(r8, bus);
            } else if ir & 0b11111000 == 0b00111000 {
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.srl_r8(r8, bus);
            } else if ir & 0b11000000 == 0b01000000 {
                let b3 = parse_operand(ir, 3, OperandType::B3);
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.bit_b3_r8(b3, r8, bus);
            } else if ir & 0b11000000 == 0b10000000 {
                let b3 = parse_operand(ir, 3, OperandType::B3);
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.res_b3_r8(b3, r8, bus);
            } else if ir & 0b11000000 == 0b11000000 {
                let b3 = parse_operand(ir, 3, OperandType::B3);
                let r8 = decode_r8(parse_operand(ir, 0, OperandType::R8));
                self.set_b3_r8(b3, r8, bus);
            }
        }
    }

    // TODO: Rename from cycle as can be over multiple clock cycles
    pub fn cycle(&mut self, bus: &mut Bus) -> Option<CycleInfo> {
        if self.halted {
            if bus.interrupts.enable & bus.interrupts.flag & 0b00011111 != 0 {
                self.halted = false;
            }

            return None;
        }

        let (opcode_bytes, opcode_address) = self.get_next_opcode(bus);
        let mut cycle_count = 0;

        // TODO: Would this be better at the end of function
        if self.interrupt_master_enable_pending {
            self.interrupt_master_enable = true;
            self.interrupt_master_enable_pending = false;
        }

        if self.interrupt_master_enable {
            cycle_count += self.handle_interrupts(bus);
        }

        self.registers.ir = self.fetch(bus);

        let (next_opcode_bytes, next_opcode_address) = self.get_next_opcode(bus);

        Some(CycleInfo {
            cycle_count,
            opcode_bytes,
            opcode_address,
            next_opcode_bytes,
            next_opcode_address,
            registers: self.registers,
        })
    }

    fn nop(&self) -> u8 {
        1
    }

    fn ld_r16_imm16(&mut self, r16: R16, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            self.registers.w = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 2 {
            set_r16(r16, self.registers.get_wz(), &mut self.registers);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn ld_r16mem_a(&mut self, r16mem: R16mem, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            let address = get_r16mem(r16mem, &self.registers);
            bus.write(address, self.registers.a);

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
        } else if self.cycle_counter == 1 {
            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn ld_a_r16mem(&mut self, r16mem: R16mem, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            let address = get_r16mem(r16mem, &self.registers);
            self.registers.z = bus.read(address);

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
        } else if self.cycle_counter == 1 {
            self.registers.a = self.registers.z;

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn ld_imm16_sp(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            self.registers.w = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 2 {
            bus.write(self.registers.get_wz(), (self.registers.sp & 0xFF) as u8);

            let next_wz = self.registers.get_wz().wrapping_add(1);
            self.registers.set_wz(next_wz);
        } else if self.cycle_counter == 3 {
            bus.write(self.registers.get_wz(), (self.registers.sp >> 8) as u8);
        } else if self.cycle_counter == 4 {
            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn inc_r16(&mut self, r16: R16, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            let value = get_r16(r16, &mut self.registers).wrapping_add(1);
            set_r16(r16, value, &mut self.registers);
        } else if self.cycle_counter == 1 {
            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn dec_r16(&mut self, r16: R16, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            let value = get_r16(r16, &mut self.registers).wrapping_sub(1);
            set_r16(r16, value, &mut self.registers);
        } else if self.cycle_counter == 1 {
            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn add_hl_r16(&mut self, r16: R16, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            let value = (get_r16(r16, &self.registers) & 0xFF) as u8;
            let (result, carry) = self.registers.l.overflowing_add(value);
            let h = (self.registers.l & 0xF) + (value & 0xF) > 0xF;

            self.registers.l = result;

            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, h);
            self.registers.set_flag(Flag::C, carry);
        } else if self.cycle_counter == 1 {
            let value = (get_r16(r16, &self.registers) >> 8) as u8;
            let (result, carry1) = self.registers.h.overflowing_add(value);
            let (result, carry2) = result.overflowing_add(value);
            let l_carry = self.registers.get_flag(Flag::C) as u8;
            let h = (self.registers.h & 0xF) + (value & 0xF) + l_carry > 0xF;

            self.registers.h = result;

            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, h);
            self.registers.set_flag(Flag::C, carry1 || carry2);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn inc_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &mut self.registers, bus);
                let result = value.wrapping_add(1);
                set_r8(r8, value, &mut self.registers, bus);

                let h = (value & 0xF) + 1 > 0xF;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, h);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 1 {
                let result = self.registers.z.wrapping_add(1);
                let hl = self.registers.get_register16(Register16::HL);
                bus.write(hl, result);

                let h = (self.registers.z & 0xF) + 1 > 0xF;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, h);
            } else if self.cycle_counter == 2 {
                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn dec_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &mut self.registers, bus);
                let result = value.wrapping_sub(1);
                set_r8(r8, value, &mut self.registers, bus);

                let h = (value & 0xF) == 0;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, h);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 1 {
                let result = self.registers.z.wrapping_sub(1);
                let hl = self.registers.get_register16(Register16::HL);
                bus.write(hl, result);

                let h = (self.registers.z & 0xF) == 0;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, h);
            } else if self.cycle_counter == 2 {
                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn ld_r8_imm8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                self.registers.z = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            } else if self.cycle_counter == 1 {
                set_r8(r8, self.registers.z, &mut self.registers, bus);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                self.registers.z = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            } else if self.cycle_counter == 1 {
                let hl = self.registers.get_register16(Register16::HL);
                bus.write(hl, self.registers.z)
            } else if self.cycle_counter == 2 {
                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn rlca(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            let c = self.registers.a >> 7 == 1;
            self.registers.a = self.registers.a.rotate_left(1);

            self.registers.set_flag(Flag::Z, false);
            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, false);
            self.registers.set_flag(Flag::C, c);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn rrca(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            let c = self.registers.a & 1 == 1;
            self.registers.a = self.registers.a.rotate_right(1);

            self.registers.set_flag(Flag::Z, false);
            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, false);
            self.registers.set_flag(Flag::C, c);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn rla(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            let c = self.registers.a >> 7 == 1;
            let mut result = self.registers.a << 1;
            result |= self.registers.get_flag(Flag::C) as u8;
            self.registers.a = result;

            self.registers.set_flag(Flag::Z, false);
            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, false);
            self.registers.set_flag(Flag::C, c);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn rra(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            let c = self.registers.a & 1 == 1;
            let mut result = self.registers.a >> 1;
            result |= (self.registers.get_flag(Flag::C) as u8) << 7;
            self.registers.a = result;

            self.registers.set_flag(Flag::Z, false);
            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, false);
            self.registers.set_flag(Flag::C, c);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn daa(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            let a = self.registers.get_register8(Register8::A);
            let half_carry = self.registers.get_flag(Flag::H);
            let carry = self.registers.get_flag(Flag::C);

            if self.registers.get_flag(Flag::N) {
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

                if half_carry || (a & 0xF) > 0x9 {
                    adjustment += 0x6;
                }

                if carry || a > 0x99 {
                    adjustment += 0x60;
                    self.registers.set_flag(Flag::C, true);
                }

                self.registers
                    .set_register8(Register8::A, a.wrapping_add(adjustment));
            }

            self.registers
                .set_flag(Flag::Z, self.registers.get_register8(Register8::A) == 0);
            self.registers.set_flag(Flag::H, false);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn cpl(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.a = !self.registers.a;

            self.registers.set_flag(Flag::N, true);
            self.registers.set_flag(Flag::H, true);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn scf(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, false);
            self.registers.set_flag(Flag::C, true);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn ccf(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, false);

            let c = self.registers.get_flag(Flag::C);
            self.registers.set_flag(Flag::C, !c);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn jr_imm8(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            let sign = self.registers.z >> 7 == 1;

            let pc = self.registers.pc;
            let (result, carry) = self.registers.z.overflowing_add((pc & 0xFF) as u8);
            self.registers.z = result;

            let adj: i8 = if carry && !sign {
                1
            } else if !carry && sign {
                -1
            } else {
                0
            };

            self.registers.w = ((pc >> 8) as u8).wrapping_add_signed(adj);
        } else if self.cycle_counter == 2 {
            self.registers.pc = self.registers.get_wz().wrapping_add(1);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn jr_cond_imm8(&mut self, cond: Cond, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else {
            if get_cond(cond, &self.registers) {
                if self.cycle_counter == 1 {
                    let sign = self.registers.z >> 7 == 1;

                    let pc = self.registers.pc;
                    let (result, carry) = self.registers.z.overflowing_add((pc & 0xFF) as u8);
                    self.registers.z = result;

                    let adj: i8 = if carry && !sign {
                        1
                    } else if !carry && sign {
                        -1
                    } else {
                        0
                    };

                    self.registers.w = ((pc >> 8) as u8).wrapping_add_signed(adj);
                } else if self.cycle_counter == 2 {
                    self.registers.ir = bus.read(self.registers.pc);
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                }
            } else {
                if self.cycle_counter == 1 {
                    self.registers.pc = self.registers.get_wz().wrapping_add(1);

                    self.registers.ir = bus.read(self.registers.pc);
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                }
            }
        }
    }

    // TODO: Implement this, use fetch_cycle for others that check for interrupts
    fn stop(&mut self, bus: &mut Bus) {
        // let interrupt_enable = bus.read(0xFFFF);
        // let interrupt_flag = bus.read(0xFF0F);
        //
        // if self.interrupt_master_enable {
        //     if interrupt_enable & interrupt_flag & 0x1F == 0 {
        //     } else {
        //     }
        // }
        //
        // 1
    }

    // Excludes ld [hl], [hl]
    fn ld_r8_r8(&mut self, r8a: R8, r8b: R8, bus: &mut Bus) {
        if matches!(r8a, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8b, &self.registers, bus);
                let hl = self.registers.get_register16(Register16::HL);
                bus.write(hl, value);
            } else if self.cycle_counter == 1 {
                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else if matches!(r8b, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 1 {
                set_r8(r8a, self.registers.z, &mut self.registers, bus);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let value = get_r8(r8b, &self.registers, bus);
                set_r8(r8a, value, &mut self.registers, bus);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    // TODO: Implement this
    fn halt(&mut self) {
        self.halted = true;
    }

    fn add_a_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let h = (self.registers.a & 0xF) + (value & 0xF) > 0xF;
                let (result, carry) = self.registers.a.overflowing_add(value);
                self.registers.a = result;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, h);
                self.registers.set_flag(Flag::C, carry);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 1 {
                let h = (self.registers.a & 0xF) + (self.registers.z & 0xF) > 0xF;
                let (result, carry) = self.registers.a.overflowing_add(self.registers.z);
                self.registers.a = result;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, h);
                self.registers.set_flag(Flag::C, carry);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn adc_a_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let carry_value = self.registers.get_flag(Flag::C) as u8;

                let h = (self.registers.a & 0xF) + (value & 0xF) + carry_value > 0xF;
                let (result, carry1) = self.registers.a.overflowing_add(value);
                let (result, carry2) = result.overflowing_add(carry_value);
                self.registers.a = result;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, h);
                self.registers.set_flag(Flag::C, carry1 || carry2);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 1 {
                let carry_value = self.registers.get_flag(Flag::C) as u8;

                let h = (self.registers.a & 0xF) + (self.registers.z & 0xF) + carry_value > 0xF;
                let (result, carry1) = self.registers.a.overflowing_add(self.registers.z);
                let (result, carry2) = result.overflowing_add(carry_value);
                self.registers.a = result;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, h);
                self.registers.set_flag(Flag::C, carry1 || carry2);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn sub_a_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let h = (self.registers.a & 0xF) < (value & 0xF);
                let (result, carry) = self.registers.a.overflowing_sub(value);
                self.registers.a = result;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, true);
                self.registers.set_flag(Flag::H, h);
                self.registers.set_flag(Flag::C, carry);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 1 {
                let h = (self.registers.a & 0xF) < (self.registers.z & 0xF);
                let (result, carry) = self.registers.a.overflowing_sub(self.registers.z);
                self.registers.a = result;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, true);
                self.registers.set_flag(Flag::H, h);
                self.registers.set_flag(Flag::C, carry);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }

        if let R8::MemoryHL = r8 { 2 } else { 1 }
    }

    fn sbc_a_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let carry_value = self.registers.get_flag(Flag::C) as u8;

                let h = (self.registers.a & 0xF) < (value & 0xF) + carry_value;
                let (result, carry1) = self.registers.a.overflowing_sub(value);
                let (result, carry2) = result.overflowing_sub(carry_value);
                self.registers.a = result;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, true);
                self.registers.set_flag(Flag::H, h);
                self.registers.set_flag(Flag::C, carry1 || carry2);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 1 {
                let carry_value = self.registers.get_flag(Flag::C) as u8;

                let h = (self.registers.a & 0xF) < (self.registers.z & 0xF) + carry_value;
                let (result, carry1) = self.registers.a.overflowing_sub(self.registers.z);
                let (result, carry2) = result.overflowing_sub(carry_value);
                self.registers.a = result;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, true);
                self.registers.set_flag(Flag::H, h);
                self.registers.set_flag(Flag::C, carry1 || carry2);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn and_a_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let result = self.registers.a & value;
                self.registers.a = result;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, true);
                self.registers.set_flag(Flag::C, false);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 1 {
                let result = self.registers.a & self.registers.z;
                self.registers.a = result;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, true);
                self.registers.set_flag(Flag::C, false);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn xor_a_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let result = self.registers.a ^ value;
                self.registers.a = result;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, false);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 1 {
                let result = self.registers.a ^ self.registers.z;
                self.registers.a = result;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, false);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn or_a_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let result = self.registers.a | value;
                self.registers.a = result;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, false);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 1 {
                let result = self.registers.a | self.registers.z;
                self.registers.a = result;

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, false);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn cp_a_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let h = (self.registers.a & 0xF) < (value & 0xF);
                let (result, c) = self.registers.a.overflowing_sub(value);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, true);
                self.registers.set_flag(Flag::H, h);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 1 {
                let h = (self.registers.a & 0xF) < (self.registers.z & 0xF);
                let (result, c) = self.registers.a.overflowing_sub(self.registers.z);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, true);
                self.registers.set_flag(Flag::H, h);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn add_a_imm8(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            let h = (self.registers.a & 0xF) + (self.registers.z & 0xF) > 0xF;
            let (result, carry) = self.registers.a.overflowing_add(self.registers.z);
            self.registers.a = result;

            self.registers.set_flag(Flag::Z, result == 0);
            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, h);
            self.registers.set_flag(Flag::C, carry);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn adc_a_imm8(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            let carry_value = self.registers.get_flag(Flag::C) as u8;

            let h = (self.registers.a & 0xF) + (self.registers.z & 0xF) + carry_value > 0xF;
            let (result, carry1) = self.registers.a.overflowing_add(self.registers.z);
            let (result, carry2) = result.overflowing_add(carry_value);
            self.registers.a = result;

            self.registers.set_flag(Flag::Z, result == 0);
            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, h);
            self.registers.set_flag(Flag::C, carry1 || carry2);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn sub_a_imm8(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            let h = (self.registers.a & 0xF) < (self.registers.z & 0xF);
            let (result, carry) = self.registers.a.overflowing_sub(self.registers.z);
            self.registers.a = result;

            self.registers.set_flag(Flag::Z, result == 0);
            self.registers.set_flag(Flag::N, true);
            self.registers.set_flag(Flag::H, h);
            self.registers.set_flag(Flag::C, carry);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn sbc_a_imm8(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            let carry_value = self.registers.get_flag(Flag::C) as u8;

            let h = (self.registers.a & 0xF) < (self.registers.z & 0xF) + carry_value;
            let (result, carry1) = self.registers.a.overflowing_sub(self.registers.z);
            let (result, carry2) = result.overflowing_sub(carry_value);
            self.registers.a = result;

            self.registers.set_flag(Flag::Z, result == 0);
            self.registers.set_flag(Flag::N, true);
            self.registers.set_flag(Flag::H, h);
            self.registers.set_flag(Flag::C, carry1 || carry2);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn and_a_imm8(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            let result = self.registers.a & self.registers.z;
            self.registers.a = result;

            self.registers.set_flag(Flag::Z, result == 0);
            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, true);
            self.registers.set_flag(Flag::C, false);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn xor_a_imm8(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            let result = self.registers.a ^ self.registers.z;
            self.registers.a = result;

            self.registers.set_flag(Flag::Z, result == 0);
            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, false);
            self.registers.set_flag(Flag::C, false);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn or_a_imm8(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            let result = self.registers.a | self.registers.z;
            self.registers.a = result;

            self.registers.set_flag(Flag::Z, result == 0);
            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, false);
            self.registers.set_flag(Flag::C, false);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn cp_a_imm8(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            let h = (self.registers.a & 0xF) < (self.registers.z & 0xF);
            let (result, c) = self.registers.a.overflowing_sub(self.registers.z);

            self.registers.set_flag(Flag::Z, result == 0);
            self.registers.set_flag(Flag::N, true);
            self.registers.set_flag(Flag::H, h);
            self.registers.set_flag(Flag::C, c);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn ret_cond(&mut self, cond: Cond, bus: &mut Bus) {
        if self.cycle_counter == 0 {
        } else {
            if get_cond(cond, &self.registers) {
                if self.cycle_counter == 1 {
                    self.registers.z = bus.read(self.registers.sp);
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                } else if self.cycle_counter == 2 {
                    self.registers.w = bus.read(self.registers.sp);
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                } else if self.cycle_counter == 3 {
                    self.registers.pc = self.registers.get_wz();
                } else if self.cycle_counter == 4 {
                    self.registers.ir = bus.read(self.registers.pc);
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                }
            } else {
                if self.cycle_counter == 1 {
                    self.registers.ir = bus.read(self.registers.pc);
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                }
            }
        }
    }

    fn ret(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.sp);
            self.registers.sp = self.registers.sp.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            self.registers.w = bus.read(self.registers.sp);
            self.registers.sp = self.registers.sp.wrapping_add(1);
        } else if self.cycle_counter == 2 {
            self.registers.pc = self.registers.get_wz();
        } else if self.cycle_counter == 3 {
            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn reti(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.sp);
            self.registers.sp = self.registers.sp.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            self.registers.w = bus.read(self.registers.sp);
            self.registers.sp = self.registers.sp.wrapping_add(1);
        } else if self.cycle_counter == 2 {
            self.registers.pc = self.registers.get_wz();
            self.interrupt_master_enable = true;
        } else if self.cycle_counter == 3 {
            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn jp_cond_imm16(&mut self, cond: Cond, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            self.registers.w = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else {
            if get_cond(cond, &self.registers) {
                if self.cycle_counter == 2 {
                    self.registers.pc = self.registers.get_wz();
                } else if self.cycle_counter == 3 {
                    self.registers.ir = bus.read(self.registers.pc);
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                }
            } else {
                if self.cycle_counter == 2 {
                    self.registers.ir = bus.read(self.registers.pc);
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                }
            }
        }
    }

    fn jp_imm16(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            self.registers.w = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 2 {
            self.registers.pc = self.registers.get_wz();
        } else if self.cycle_counter == 3 {
            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn jp_hl(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.pc = self.registers.get_register16(Register16::HL) + 1;

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn call_cond_imm16(&mut self, cond: Cond, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.sp);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            self.registers.w = bus.read(self.registers.sp);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else {
            if get_cond(cond, &self.registers) {
                if self.cycle_counter == 2 {
                    self.registers.sp = self.registers.sp.wrapping_sub(1);
                } else if self.cycle_counter == 3 {
                    let value = (self.registers.pc >> 8) as u8;
                    bus.write(self.registers.sp, value);
                    self.registers.sp = self.registers.sp.wrapping_sub(1);
                } else if self.cycle_counter == 4 {
                    let value = (self.registers.pc & 0xFF) as u8;
                    bus.write(self.registers.sp, value);
                    self.registers.pc = self.registers.get_wz();
                } else if self.cycle_counter == 5 {
                    self.registers.ir = bus.read(self.registers.pc);
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                }
            } else {
                if self.cycle_counter == 2 {
                    self.registers.ir = bus.read(self.registers.pc);
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                }
            }
        }
    }

    fn call_imm16(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            self.registers.w = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 2 {
            self.registers.sp = self.registers.sp.wrapping_sub(1);
        } else if self.cycle_counter == 3 {
            let value = (self.registers.pc >> 8) as u8;
            bus.write(self.registers.sp, value);
            self.registers.sp = self.registers.sp.wrapping_sub(1);
        } else if self.cycle_counter == 4 {
            let value = (self.registers.pc & 0xFF) as u8;
            bus.write(self.registers.sp, value);
            self.registers.pc = self.registers.get_wz();
        } else if self.cycle_counter == 5 {
            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn rst_tgt3(&mut self, tgt3: u8, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.sp = self.registers.sp.wrapping_sub(1);
        } else if self.cycle_counter == 1 {
            let value = (self.registers.pc >> 8) as u8;
            bus.write(self.registers.sp, value);
            self.registers.sp = self.registers.sp.wrapping_sub(1);
        } else if self.cycle_counter == 2 {
            let value = (self.registers.pc & 0xFF) as u8;
            bus.write(self.registers.sp, value);
            self.registers.pc = 8 * (tgt3 as u16);
        } else if self.cycle_counter == 3 {
            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn pop_r16stk(&mut self, r16stk: R16stk, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.sp);
            self.registers.sp = self.registers.sp.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            self.registers.w = bus.read(self.registers.sp);
            self.registers.sp = self.registers.sp.wrapping_add(1);
        } else if self.cycle_counter == 2 {
            let value = get_r16stk(r16stk, &self.registers);
            set_r16stk(r16stk, value, &mut self.registers);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn push_r16stk(&mut self, r16stk: R16stk, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.sp = self.registers.sp.wrapping_sub(1);
        } else if self.cycle_counter == 1 {
            let value = (get_r16stk(r16stk, &self.registers) >> 8) as u8;
            bus.write(self.registers.sp, value);
            self.registers.sp = self.registers.sp.wrapping_sub(1);
        } else if self.cycle_counter == 2 {
            let value = (get_r16stk(r16stk, &self.registers) & 0xFF) as u8;
            bus.write(self.registers.sp, value);
        } else if self.cycle_counter == 3 {
            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn rlc_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let c = value >> 7 == 1;
                let result = value.rotate_left(1);
                set_r8(r8, result, &mut self.registers, bus);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 0 {
                let c = self.registers.z >> 7 == 1;
                let result = self.registers.z.rotate_left(1);

                let hl = self.registers.get_register16(Register16::HL);
                bus.write(hl, result);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn rrc_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let c = value & 1 == 1;
                let result = value.rotate_right(1);
                set_r8(r8, result, &mut self.registers, bus);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 0 {
                let c = self.registers.z & 1 == 1;
                let result = self.registers.z.rotate_right(1);

                let hl = self.registers.get_register16(Register16::HL);
                bus.write(hl, result);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn rl_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let c = value >> 7 == 1;
                let mut result = value << 1;
                result |= self.registers.get_flag(Flag::C) as u8;
                set_r8(r8, result, &mut self.registers, bus);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 0 {
                let c = self.registers.z >> 7 == 1;
                let mut result = self.registers.z << 1;
                result |= self.registers.get_flag(Flag::C) as u8;

                let hl = self.registers.get_register16(Register16::HL);
                bus.write(hl, result);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn rr_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let c = value & 1 == 1;
                let mut result = value >> 1;
                result |= (self.registers.get_flag(Flag::C) as u8) << 7;
                set_r8(r8, result, &mut self.registers, bus);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 0 {
                let c = self.registers.z & 1 == 1;
                let mut result = self.registers.z >> 1;
                result |= (self.registers.get_flag(Flag::C) as u8) << 7;

                let hl = self.registers.get_register16(Register16::HL);
                bus.write(hl, result);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn sla_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let c = value >> 7 == 1;
                let result = value << 1;
                set_r8(r8, result, &mut self.registers, bus);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 0 {
                let c = self.registers.z >> 7 == 1;
                let result = self.registers.z << 1;

                let hl = self.registers.get_register16(Register16::HL);
                bus.write(hl, result);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn sra_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let c = value & 1 == 1;
                let mut result = value >> 1;
                result |= value & (1 << 7);
                set_r8(r8, result, &mut self.registers, bus);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 0 {
                let c = self.registers.z & 1 == 1;
                let mut result = self.registers.z >> 1;
                result |= self.registers.z & (1 << 7);

                let hl = self.registers.get_register16(Register16::HL);
                bus.write(hl, result);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn swap_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let result = (value << 4) | (value >> 4);
                set_r8(r8, result, &mut self.registers, bus);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, false);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 1 {
                let result = (self.registers.z << 4) | (self.registers.z >> 4);

                let hl = self.registers.get_register16(Register16::HL);
                bus.write(hl, result);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, false);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn srl_r8(&mut self, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let c = value & 1 == 1;
                let result = value >> 1;
                set_r8(r8, result, &mut self.registers, bus);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 0 {
                let c = self.registers.z & 1 == 1;
                let result = self.registers.z >> 1;

                let hl = self.registers.get_register16(Register16::HL);
                bus.write(hl, result);

                self.registers.set_flag(Flag::Z, result == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, false);
                self.registers.set_flag(Flag::C, c);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn bit_b3_r8(&mut self, b3: u8, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let bit = (value >> b3) & 1;

                self.registers.set_flag(Flag::Z, bit == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, true);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 0 {
                let bit = (self.registers.z >> b3) & 1;

                self.registers.set_flag(Flag::Z, bit == 0);
                self.registers.set_flag(Flag::N, false);
                self.registers.set_flag(Flag::H, true);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn res_b3_r8(&mut self, b3: u8, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let result = value & !(1 << b3);
                set_r8(r8, result, &mut self.registers, bus);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 0 {
                let result = self.registers.z & !(1 << b3);
                set_r8(r8, result, &mut self.registers, bus);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn set_b3_r8(&mut self, b3: u8, r8: R8, bus: &mut Bus) {
        if !matches!(r8, R8::MemoryHL) {
            if self.cycle_counter == 0 {
                let value = get_r8(r8, &self.registers, bus);
                let result = value | (1 << b3);
                set_r8(r8, result, &mut self.registers, bus);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        } else {
            if self.cycle_counter == 0 {
                let hl = self.registers.get_register16(Register16::HL);
                self.registers.z = bus.read(hl);
            } else if self.cycle_counter == 0 {
                let result = self.registers.z | (1 << b3);
                set_r8(r8, result, &mut self.registers, bus);

                self.registers.ir = bus.read(self.registers.pc);
                self.registers.pc = self.registers.pc.wrapping_add(1);
            }
        }
    }

    fn ldh_c_a(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            let address = 0xFF | (self.registers.c as u16);
            bus.write(address, self.registers.a);
        } else if self.cycle_counter == 1 {
            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn ldh_imm8_a(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            let address = 0xFF | (self.registers.z as u16);
            bus.write(address, self.registers.a);
        } else if self.cycle_counter == 2 {
            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn ld_imm16_a(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            self.registers.w = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 2 {
            bus.write(self.registers.get_wz(), self.registers.a);
        } else if self.cycle_counter == 3 {
            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn ldh_a_c(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            let address = 0xFF | (self.registers.c as u16);
            self.registers.z = bus.read(address);
        } else if self.cycle_counter == 1 {
            self.registers.a = self.registers.z;

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn ldh_a_imm8(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            let address = 0xFF | (self.registers.z as u16);
            self.registers.z = bus.read(address);
        } else if self.cycle_counter == 2 {
            self.registers.a = self.registers.z;

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn ld_a_imm16(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            self.registers.w = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 2 {
            self.registers.z = bus.read(self.registers.get_wz());
        } else if self.cycle_counter == 3 {
            self.registers.a = self.registers.z;

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    // TODO: Last two to do here!
    fn add_sp_imm8(&mut self, bus: &mut Bus) -> u8 {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            let sign = self.registers.z >> 7 == 1;
            let sp = self.registers.sp;
            let h = ((self.registers.sp & 0xF) as u8) + (self.registers.z & 0xF) > 0xF;
            let (result, carry) = ((sp & 0xFF) as u8).overflowing_add(self.registers.z);
            self.registers.z = result;

            self.registers.set_flag(Flag::Z, result == 0);
            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, h);
            self.registers.set_flag(Flag::C, carry);
        } else if self.cycle_counter == 2 {
            let h = (self.registers.a & 0xF) + (self.registers.z & 0xF) > 0xF;
            let (result, carry) = self.registers.a.overflowing_add(self.registers.z);
            self.registers.a = result;

            self.registers.set_flag(Flag::Z, result == 0);
            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, h);
            self.registers.set_flag(Flag::C, carry);

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }

        let sp = self.registers.get_register16(Register16::SP);
        let value = self.fetch(bus) as i8;
        let result = sp.wrapping_add_signed(value as i16);

        self.registers.set_register16(Register16::SP, result);

        let half_carry = (sp & 0xF) + ((value as u8 as u16) & 0xF) > 0xF;
        let carry = (sp & 0xFF) + ((value as u8 as u16) & 0xFF) > 0xFF;

        self.registers.set_flag(Flag::Z, false);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, half_carry);
        self.registers.set_flag(Flag::C, carry);

        4
    }

    fn ld_hl_sp_imm8(&mut self, bus: &mut Bus) -> u8 {
        if self.cycle_counter == 0 {
            self.registers.z = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        } else if self.cycle_counter == 1 {
            self.registers.set_flag(Flag::Z, false);
            self.registers.set_flag(Flag::N, false);
            self.registers.set_flag(Flag::H, half_carry);
            self.registers.set_flag(Flag::C, carry);
        } else if self.cycle_counter == 2 {
            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }

        let value = self.fetch(bus) as i8;
        let sp = self.registers.get_register16(Register16::SP);

        self.registers
            .set_register16(Register16::HL, sp.wrapping_add_signed(value as i16));

        let half_carry = (sp & 0xF) + ((value as u8 as u16) & 0xF) > 0xF;
        let carry = (sp & 0xFF) + ((value as u8 as u16) & 0xFF) > 0xFF;

        self.registers.set_flag(Flag::Z, false);
        self.registers.set_flag(Flag::N, false);
        self.registers.set_flag(Flag::H, half_carry);
        self.registers.set_flag(Flag::C, carry);

        3
    }

    fn ld_sp_hl(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.registers.sp = self.registers.get_register16(Register16::HL);
        } else if self.cycle_counter == 1 {
            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn di(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.interrupt_master_enable = false;
            self.interrupt_master_enable_pending = false;

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }

    fn ei(&mut self, bus: &mut Bus) {
        if self.cycle_counter == 0 {
            self.interrupt_master_enable_pending = true;

            self.registers.ir = bus.read(self.registers.pc);
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }
    }
}
