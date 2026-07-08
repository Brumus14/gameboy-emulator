use std::fmt::format;

use crate::core::{
    cpu,
    gameboy::CycleInfo,
    opcodes::{
        OperandType, decode_cond, decode_r8, decode_r16, decode_r16mem, decode_r16stk,
        parse_operand,
    },
};

pub struct Debug;

impl Debug {
    pub fn opcode_to_string(bytes: [u8; 3]) -> String {
        if bytes[0] == 0b00000000 {
            "nop".to_string()
        } else if bytes[0] & 0b11001111 == 0b00000001 {
            let r16 = decode_r16(parse_operand(bytes[0], 4, OperandType::R16));
            let imm16 = merge_bytes(bytes[1], bytes[2]);
            format!("ld {}, {:04X}", r16.to_string(), imm16)
        } else if bytes[0] & 0b11001111 == 0b00000010 {
            let r16mem = decode_r16mem(parse_operand(bytes[0], 4, OperandType::R16mem));
            format!("ld [{}], a", r16mem.to_string())
        } else if bytes[0] & 0b11001111 == 0b00001010 {
            let r16mem = decode_r16mem(parse_operand(bytes[0], 4, OperandType::R16mem));
            format!("ld a, [{}]", r16mem.to_string())
        } else if bytes[0] == 0b00001000 {
            let imm16 = merge_bytes(bytes[1], bytes[2]);
            format!("ld [{:04X}], sp", imm16)
        } else if bytes[0] & 0b11001111 == 0b00000011 {
            let r16 = decode_r16(parse_operand(bytes[0], 4, OperandType::R16));
            format!("inc {}", r16.to_string())
        } else if bytes[0] & 0b11001111 == 0b00001011 {
            let r16 = decode_r16(parse_operand(bytes[0], 4, OperandType::R16));
            format!("dec {}", r16.to_string())
        } else if bytes[0] & 0b11001111 == 0b00001001 {
            let r16 = decode_r16(parse_operand(bytes[0], 4, OperandType::R16));
            format!("add hl, {}", r16.to_string())
        } else if bytes[0] & 0b11000111 == 0b00000100 {
            let r8 = decode_r8(parse_operand(bytes[0], 3, OperandType::R8));
            format!("inc {}", r8.to_string())
        } else if bytes[0] & 0b11000111 == 0b00000101 {
            let r8 = decode_r8(parse_operand(bytes[0], 3, OperandType::R8));
            format!("dec {}", r8.to_string())
        } else if bytes[0] & 0b11000111 == 0b00000110 {
            let r8 = decode_r8(parse_operand(bytes[0], 3, OperandType::R8));
            let imm8 = bytes[1];
            format!("ld {}, {:02X}", r8.to_string(), imm8)
        } else if bytes[0] == 0b00000111 {
            "rlca".to_string()
        } else if bytes[0] == 0b00001111 {
            "rrca".to_string()
        } else if bytes[0] == 0b00010111 {
            "rla".to_string()
        } else if bytes[0] == 0b00011111 {
            "rra".to_string()
        } else if bytes[0] == 0b00100111 {
            "daa".to_string()
        } else if bytes[0] == 0b00101111 {
            "cpl".to_string()
        } else if bytes[0] == 0b00110111 {
            "scf".to_string()
        } else if bytes[0] == 0b00111111 {
            "ccf".to_string()
        } else if bytes[0] == 0b00011000 {
            let imm8 = bytes[1] as i8;

            if imm8 >= 0 {
                format!("jr {:02X}", imm8)
            } else {
                format!("jr -{:02X}", imm8.abs())
            }
        } else if bytes[0] & 0b11100111 == 0b00100000 {
            let cond = decode_cond(parse_operand(bytes[0], 3, OperandType::Cond));

            let imm8 = bytes[1] as i8;

            if imm8 >= 0 {
                format!("jr {}, {:02X}", cond.to_string(), imm8)
            } else {
                format!("jr {}, -{:02X}", cond.to_string(), imm8.abs())
            }
        } else if bytes[0] == 0b00010000 {
            "stop".to_string()
        } else if bytes[0] & 0b11000000 == 0b01000000 && bytes[0] != 0b01110110 {
            let r8a = decode_r8(parse_operand(bytes[0], 3, OperandType::R8));
            let r8b = decode_r8(parse_operand(bytes[0], 0, OperandType::R8));
            format!("ld {}, {}", r8a.to_string(), r8b.to_string())
        } else if bytes[0] == 0b01110110 {
            "halt".to_string()
        } else if bytes[0] & 0b11111000 == 0b10000000 {
            let r8 = decode_r8(parse_operand(bytes[0], 0, OperandType::R8));
            format!("add a, {}", r8.to_string())
        } else if bytes[0] & 0b11111000 == 0b10001000 {
            let r8 = decode_r8(parse_operand(bytes[0], 0, OperandType::R8));
            format!("adc a, {}", r8.to_string())
        } else if bytes[0] & 0b11111000 == 0b10010000 {
            let r8 = decode_r8(parse_operand(bytes[0], 0, OperandType::R8));
            format!("sub a, {}", r8.to_string())
        } else if bytes[0] & 0b11111000 == 0b10011000 {
            let r8 = decode_r8(parse_operand(bytes[0], 0, OperandType::R8));
            format!("sbc a, {}", r8.to_string())
        } else if bytes[0] & 0b11111000 == 0b10100000 {
            let r8 = decode_r8(parse_operand(bytes[0], 0, OperandType::R8));
            format!("and a, {}", r8.to_string())
        } else if bytes[0] & 0b11111000 == 0b10101000 {
            let r8 = decode_r8(parse_operand(bytes[0], 0, OperandType::R8));
            format!("xor a, {}", r8.to_string())
        } else if bytes[0] & 0b11111000 == 0b10110000 {
            let r8 = decode_r8(parse_operand(bytes[0], 0, OperandType::R8));
            format!("or a, {}", r8.to_string())
        } else if bytes[0] & 0b11111000 == 0b10111000 {
            let r8 = decode_r8(parse_operand(bytes[0], 0, OperandType::R8));
            format!("cp a, {}", r8.to_string())
        } else if bytes[0] == 0b11000110 {
            let imm8 = bytes[1];
            format!("add a, {:02X}", imm8)
        } else if bytes[0] == 0b11001110 {
            let imm8 = bytes[1];
            format!("adc a, {:02X}", imm8)
        } else if bytes[0] == 0b11010110 {
            let imm8 = bytes[1];
            format!("sub a, {:02X}", imm8)
        } else if bytes[0] == 0b11011110 {
            let imm8 = bytes[1];
            format!("sbc a, {:02X}", imm8)
        } else if bytes[0] == 0b11100110 {
            let imm8 = bytes[1];
            format!("and a, {:02X}", imm8)
        } else if bytes[0] == 0b11101110 {
            let imm8 = bytes[1];
            format!("xor a, {:02X}", imm8)
        } else if bytes[0] == 0b11110110 {
            let imm8 = bytes[1];
            format!("or a, {:02X}", imm8)
        } else if bytes[0] == 0b11111110 {
            let imm8 = bytes[1];
            format!("cp a, {:02X}", imm8)
        } else if bytes[0] & 0b11100111 == 0b11000000 {
            let cond = decode_cond(parse_operand(bytes[0], 3, OperandType::Cond));
            format!("ret {}", cond.to_string())
        } else if bytes[0] == 0b11001001 {
            "ret".to_string()
        } else if bytes[0] == 0b11011001 {
            "reti".to_string()
        } else if bytes[0] & 0b11100111 == 0b11000010 {
            let cond = decode_cond(parse_operand(bytes[0], 3, OperandType::Cond));
            let imm16 = merge_bytes(bytes[1], bytes[2]);
            format!("jp {}, {:04X}", cond.to_string(), imm16)
        } else if bytes[0] == 0b11000011 {
            let imm16 = merge_bytes(bytes[1], bytes[2]);
            format!("jp {:04X}", imm16)
        } else if bytes[0] == 0b11101001 {
            "jp hl".to_string()
        } else if bytes[0] & 0b11100111 == 0b11000100 {
            let cond = decode_cond(parse_operand(bytes[0], 3, OperandType::Cond));
            let imm16 = merge_bytes(bytes[1], bytes[2]);
            format!("call {}, {:04X}", cond.to_string(), imm16)
        } else if bytes[0] == 0b11001101 {
            let imm16 = merge_bytes(bytes[1], bytes[2]);
            format!("call {:04X}", imm16)
        } else if bytes[0] & 0b11000111 == 0b11000111 {
            let tgt3 = parse_operand(bytes[0], 3, OperandType::Tgt3);
            format!("rst {:02X}", tgt3 * 8)
        } else if bytes[0] & 0b11001111 == 0b11000001 {
            let r16stk = decode_r16stk(parse_operand(bytes[0], 3, OperandType::R16stk));
            format!("pop {}", r16stk.to_string())
        } else if bytes[0] & 0b11001111 == 0b11000101 {
            let r16stk = decode_r16stk(parse_operand(bytes[0], 3, OperandType::R16stk));
            format!("push {}", r16stk.to_string())
        } else if bytes[0] == 0b11001011 {
            if bytes[1] & 0b11111000 == 0b00000000 {
                let r8 = decode_r8(parse_operand(bytes[1], 0, OperandType::R8));
                format!("rlc {}", r8.to_string())
            } else if bytes[1] & 0b11111000 == 0b00001000 {
                let r8 = decode_r8(parse_operand(bytes[1], 0, OperandType::R8));
                format!("rrc {}", r8.to_string())
            } else if bytes[1] & 0b11111000 == 0b00010000 {
                let r8 = decode_r8(parse_operand(bytes[1], 0, OperandType::R8));
                format!("rl {}", r8.to_string())
            } else if bytes[1] & 0b11111000 == 0b00011000 {
                let r8 = decode_r8(parse_operand(bytes[1], 0, OperandType::R8));
                format!("rr {}", r8.to_string())
            } else if bytes[1] & 0b11111000 == 0b00100000 {
                let r8 = decode_r8(parse_operand(bytes[1], 0, OperandType::R8));
                format!("sla {}", r8.to_string())
            } else if bytes[1] & 0b11111000 == 0b00101000 {
                let r8 = decode_r8(parse_operand(bytes[1], 0, OperandType::R8));
                format!("sra {}", r8.to_string())
            } else if bytes[1] & 0b11111000 == 0b00110000 {
                let r8 = decode_r8(parse_operand(bytes[1], 0, OperandType::R8));
                format!("swap {}", r8.to_string())
            } else if bytes[1] & 0b11111000 == 0b00111000 {
                let r8 = decode_r8(parse_operand(bytes[1], 0, OperandType::R8));
                format!("srl {}", r8.to_string())
            } else if bytes[1] & 0b11000000 == 0b01000000 {
                let b3 = parse_operand(bytes[1], 3, OperandType::B3);
                let r8 = decode_r8(parse_operand(bytes[1], 0, OperandType::R8));
                format!("bit {}, {}", b3, r8.to_string())
            } else if bytes[1] & 0b11000000 == 0b10000000 {
                let b3 = parse_operand(bytes[1], 3, OperandType::B3);
                let r8 = decode_r8(parse_operand(bytes[1], 0, OperandType::R8));
                format!("res {}, {}", b3, r8.to_string())
            } else if bytes[1] & 0b11000000 == 0b11000000 {
                let b3 = parse_operand(bytes[1], 3, OperandType::B3);
                let r8 = decode_r8(parse_operand(bytes[1], 0, OperandType::R8));
                format!("set {}, {}", b3, r8.to_string())
            } else {
                unreachable!()
            }
        } else if bytes[0] == 0b11100010 {
            "ldh [c], a".to_string()
        } else if bytes[0] == 0b11100000 {
            let imm8 = bytes[1];
            format!("ldh [{:02X}], a", imm8)
        } else if bytes[0] == 0b11101010 {
            let imm16 = merge_bytes(bytes[1], bytes[2]);
            format!("ld [{:04X}], a", imm16)
        } else if bytes[0] == 0b11110010 {
            "ldh a, [c]".to_string()
        } else if bytes[0] == 0b11110000 {
            let imm8 = bytes[1];
            format!("ldh a, [{:02X}]", imm8)
        } else if bytes[0] == 0b11111010 {
            let imm16 = merge_bytes(bytes[1], bytes[2]);
            format!("ld a, [{:04X}]", imm16)
        } else if bytes[0] == 0b11101000 {
            let imm8 = bytes[1] as i8;

            if imm8 >= 0 {
                format!("add sp, {:02X}", imm8)
            } else {
                format!("add sp, -{:02X}", imm8.abs())
            }
        } else if bytes[0] == 0b11111000 {
            let imm8 = bytes[1] as i8;

            if imm8 >= 0 {
                format!("jr hl, sp, {:02X}", imm8)
            } else {
                format!("jr hl, sp, -{:02X}", imm8.abs())
            }
        } else if bytes[0] == 0b11111001 {
            "ldh sp, hl".to_string()
        } else if bytes[0] == 0b11110011 {
            "di".to_string()
        } else if bytes[0] == 0b11111011 {
            "ei".to_string()
        } else {
            "invalid".to_string()
        }
    }
}

fn merge_bytes(byte1: u8, byte2: u8) -> u16 {
    (byte1 as u16) | ((byte2 as u16) << 8)
}
