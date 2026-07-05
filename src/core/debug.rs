use crate::core::{
    cpu::CycleInfo,
    opcodes::{OperandType, decode_r16, parse_operand},
};

pub struct Debug {}

impl Debug {
    pub fn trace(cycle_info: CycleInfo) {}

    // pub fn get_opcode_string(&self) -> String {
    //     if self.opcode[0] == 0b00000000 {
    //         "nop".to_string()
    //     } else if self.opcode[0] & 0b11001111 == 0b00000001 {
    //         let r16 = decode_r16(parse_operand(self.opcode[0], 4, OperandType::R16));
    //         format!("ld {}, {}", r16.to_string(), self.merge_bytes())
    //     } else {
    //         "invalid".to_string()
    //     }
    //     // else if self.opcode[0] & 0b11001111 == 0b00000010 {
    //     //     let r16mem = decode_r16mem(parse_operand(self.opcode[0], 4, OperandType::R16mem));
    //     //     self.ld_r16mem_a(r16mem, bus);
    //     // } else if self.opcode[0] & 0b11001111 == 0b00001010 {
    //     //     let r16mem = decode_r16mem(parse_operand(self.opcode[0], 4, OperandType::R16mem));
    //     //     self.ld_a_r16mem(r16mem, bus);
    //     // } else if self.opcode[0] == 0b00001000 {
    //     //     self.ld_imm16_sp(bus);
    //     // } else if self.opcode[0] & 0b11001111 == 0b00000011 {
    //     //     let r16 = decode_r16(parse_operand(self.opcode[0], 4, OperandType::R16));
    //     //     self.inc_r16(r16);
    //     // } else if self.opcode[0] & 0b11001111 == 0b00001011 {
    //     //     let r16 = decode_r16(parse_operand(self.opcode[0], 4, OperandType::R16));
    //     //     self.dec_r16(r16);
    //     // } else if self.opcode[0] & 0b11001111 == 0b00001001 {
    //     //     let r16 = decode_r16(parse_operand(self.opcode[0], 4, OperandType::R16));
    //     //     self.add_hl_r16(r16);
    //     // } else if self.opcode[0] & 0b11000111 == 0b00000100 {
    //     //     let r8 = decode_r8(parse_operand(self.opcode[0], 3, OperandType::R8));
    //     //     self.inc_r8(r8, bus);
    //     // } else if self.opcode[0] & 0b11000111 == 0b00000101 {
    //     //     let r8 = decode_r8(parse_operand(self.opcode[0], 3, OperandType::R8));
    //     //     self.dec_r8(r8, bus);
    //     // } else if self.opcode[0] & 0b11000111 == 0b00000110 {
    //     //     let r8 = decode_r8(parse_operand(self.opcode[0], 3, OperandType::R8));
    //     //     self.ld_r8_imm8(r8, bus);
    //     // } else if self.opcode[0] == 0b00000111 {
    //     //     self.rlca();
    //     // } else if self.opcode[0] == 0b00001111 {
    //     //     self.rrca();
    //     // } else if self.opcode[0] == 0b00010111 {
    //     //     self.rla();
    //     // } else if self.opcode[0] == 0b00011111 {
    //     //     self.rra();
    //     // } else if self.opcode[0] == 0b00100111 {
    //     //     self.daa();
    //     // } else if self.opcode[0] == 0b00101111 {
    //     //     self.cpl();
    //     // } else if self.opcode[0] == 0b00110111 {
    //     //     self.scf();
    //     // } else if self.opcode[0] == 0b00111111 {
    //     //     self.ccf();
    //     // } else if self.opcode[0] == 0b00011000 {
    //     //     self.jr_imm8(bus);
    //     // } else if self.opcode[0] & 0b11100111 == 0b00100000 {
    //     //     let cond = decode_cond(parse_operand(self.opcode[0], 3, OperandType::Cond));
    //     //     self.jr_cond_imm8(cond, bus);
    //     // } else if self.opcode[0] == 0b00010000 {
    //     //     todo!("stop")
    //     // } else if self.opcode[0] & 0b11100111 == 0b00100000 && self.opcode[0] != 0b01110110 {
    //     //     let r8a = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //     let r8b = decode_r8(parse_operand(self.opcode[0], 3, OperandType::R8));
    //     //     self.ld_r8_r8(r8a, r8b, bus);
    //     // } else if self.opcode[0] == 0b01110110 {
    //     //     todo!("halt")
    //     // } else if self.opcode[0] & 0b11111000 == 0b10000000 {
    //     //     let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //     self.add_a_r8(r8, bus);
    //     // } else if self.opcode[0] & 0b11111000 == 0b10001000 {
    //     //     let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //     self.adc_a_r8(r8, bus);
    //     // } else if self.opcode[0] & 0b11111000 == 0b10010000 {
    //     //     let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //     self.sub_a_r8(r8, bus);
    //     // } else if self.opcode[0] & 0b11111000 == 0b10011000 {
    //     //     let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //     self.sbc_a_r8(r8, bus);
    //     // } else if self.opcode[0] & 0b11111000 == 0b10100000 {
    //     //     let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //     self.and_a_r8(r8, bus);
    //     // } else if self.opcode[0] & 0b11111000 == 0b10101000 {
    //     //     let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //     self.xor_a_r8(r8, bus);
    //     // } else if self.opcode[0] & 0b11111000 == 0b10110000 {
    //     //     let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //     self.or_a_r8(r8, bus);
    //     // } else if self.opcode[0] & 0b11111000 == 0b10111000 {
    //     //     let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //     self.or_a_r8(r8, bus);
    //     // } else if self.opcode[0] == 0b11000110 {
    //     //     self.add_a_imm8(bus);
    //     // } else if self.opcode[0] == 0b11001110 {
    //     //     self.adc_a_imm8(bus);
    //     // } else if self.opcode[0] == 0b11010110 {
    //     //     self.sub_a_imm8(bus);
    //     // } else if self.opcode[0] == 0b11011110 {
    //     //     self.sbc_a_imm8(bus);
    //     // } else if self.opcode[0] == 0b11100110 {
    //     //     self.and_a_imm8(bus);
    //     // } else if self.opcode[0] == 0b11101110 {
    //     //     self.xor_a_imm8(bus);
    //     // } else if self.opcode[0] == 0b11110110 {
    //     //     self.or_a_imm8(bus);
    //     // } else if self.opcode[0] == 0b11111110 {
    //     //     self.cp_a_imm8(bus);
    //     // } else if self.opcode[0] & 0b11100111 == 0b11000000 {
    //     //     let cond = decode_cond(parse_operand(self.opcode[0], 3, OperandType::Cond));
    //     //     self.ret_cond(cond, bus);
    //     // } else if self.opcode[0] == 0b11001001 {
    //     //     self.ret(bus);
    //     // } else if self.opcode[0] == 0b11011001 {
    //     //     self.reti(bus);
    //     // } else if self.opcode[0] & 0b11100111 == 0b11000010 {
    //     //     let cond = decode_cond(parse_operand(self.opcode[0], 3, OperandType::Cond));
    //     //     self.jp_cond_imm16(cond, bus);
    //     // } else if self.opcode[0] == 0b11000011 {
    //     //     self.jp_imm16(bus);
    //     // } else if self.opcode[0] == 0b11101001 {
    //     //     self.jp_hl();
    //     // } else if self.opcode[0] & 0b11100111 == 0b11000100 {
    //     //     let cond = decode_cond(parse_operand(self.opcode[0], 3, OperandType::Cond));
    //     //     self.call_cond_imm16(cond, bus);
    //     // } else if self.opcode[0] == 0b11001101 {
    //     //     self.call_imm16(bus);
    //     // } else if self.opcode[0] & 0b11000111 == 0b11000111 {
    //     //     let tgt3 = parse_operand(self.opcode[0], 3, OperandType::Tgt3);
    //     //     self.rst_tgt3(tgt3, bus);
    //     // } else if self.opcode[0] & 0b11001111 == 0b11000001 {
    //     //     let r16stk = decode_r16stk(parse_operand(self.opcode[0], 3, OperandType::R16stk));
    //     //     self.pop_r16stk(r16stk, bus);
    //     // } else if self.opcode[0] & 0b11001111 == 0b11000101 {
    //     //     let r16stk = decode_r16stk(parse_operand(self.opcode[0], 3, OperandType::R16stk));
    //     //     self.push_r16stk(r16stk, bus);
    //     // } else if self.opcode[0] == 0b11001011 {
    //     //     let self.opcode[0] = self.fetch(bus);
    //     //
    //     //     if self.opcode[0] & 0b11111000 == 0b00000000 {
    //     //         let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //         self.rlc_r8(r8, bus);
    //     //     } else if self.opcode[0] & 0b11111000 == 0b00001000 {
    //     //         let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //         self.rrc_r8(r8, bus);
    //     //     } else if self.opcode[0] & 0b11111000 == 0b00010000 {
    //     //         let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //         self.rl_r8(r8, bus);
    //     //     } else if self.opcode[0] & 0b11111000 == 0b00011000 {
    //     //         let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //         self.rr_r8(r8, bus);
    //     //     } else if self.opcode[0] & 0b11111000 == 0b00100000 {
    //     //         let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //         self.sla_r8(r8, bus);
    //     //     } else if self.opcode[0] & 0b11111000 == 0b00101000 {
    //     //         let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //         self.sra_r8(r8, bus);
    //     //     } else if self.opcode[0] & 0b11111000 == 0b00110000 {
    //     //         let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //         self.swap_r8(r8, bus);
    //     //     } else if self.opcode[0] & 0b11111000 == 0b00111000 {
    //     //         let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //         self.srl_r8(r8, bus);
    //     //     } else if self.opcode[0] & 0b11000000 == 0b01000000 {
    //     //         let b3 = parse_operand(self.opcode[0], 3, OperandType::B3);
    //     //         let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //         self.bit_b3_r8(b3, r8, bus);
    //     //     } else if self.opcode[0] & 0b11000000 == 0b10000000 {
    //     //         let b3 = parse_operand(self.opcode[0], 3, OperandType::B3);
    //     //         let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //         self.res_b3_r8(b3, r8, bus);
    //     //     } else if self.opcode[0] & 0b11000000 == 0b11000000 {
    //     //         let b3 = parse_operand(self.opcode[0], 3, OperandType::B3);
    //     //         let r8 = decode_r8(parse_operand(self.opcode[0], 0, OperandType::R8));
    //     //         self.set_b3_r8(b3, r8, bus);
    //     //     }
    //     // } else if self.opcode[0] == 0b11100010 {
    //     //     self.ldh_c_a(bus);
    //     // } else if self.opcode[0] == 0b11100000 {
    //     //     self.ldh_imm8_a(bus);
    //     // } else if self.opcode[0] == 0b11101010 {
    //     //     self.ld_imm16_a(bus);
    //     // } else if self.opcode[0] == 0b11110010 {
    //     //     self.ldh_a_c(bus);
    //     // } else if self.opcode[0] == 0b11110000 {
    //     //     self.ldh_a_imm8(bus);
    //     // } else if self.opcode[0] == 0b11111010 {
    //     //     self.ld_a_imm16(bus);
    //     // } else if self.opcode[0] == 0b11101000 {
    //     //     self.add_sp_imm8(bus);
    //     // } else if self.opcode[0] == 0b11111000 {
    //     //     self.ld_hl_sp_imm8(bus);
    //     // } else if self.opcode[0] == 0b11111001 {
    //     //     self.ld_sp_hl();
    //     // } else if self.opcode[0] == 0b11110011 {
    //     //     self.di();
    //     // } else if self.opcode[0] == 0b11111011 {
    //     //     self.ei();
    //     // }
    // }
}
