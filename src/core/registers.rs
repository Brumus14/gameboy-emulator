#[derive(Clone, Copy)]
pub enum Register8 {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Clone, Copy)]
pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

#[derive(Clone, Copy)]
pub enum Flag {
    Zero,
    Negative,
    HalfCarry,
    Carry,
}

pub struct Registers {
    pub a: u8,
    f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0xFFFE,
            pc: 0x100,
        }
    }

    pub fn get_register8(&self, register: Register8) -> u8 {
        match register {
            Register8::A => self.a,
            Register8::F => self.f,
            Register8::B => self.b,
            Register8::C => self.c,
            Register8::D => self.d,
            Register8::E => self.e,
            Register8::H => self.h,
            Register8::L => self.l,
        }
    }

    pub fn set_register8(&mut self, register: Register8, value: u8) {
        match register {
            Register8::A => self.a = value,
            Register8::F => self.f = value,
            Register8::B => self.b = value,
            Register8::C => self.c = value,
            Register8::D => self.d = value,
            Register8::E => self.e = value,
            Register8::H => self.h = value,
            Register8::L => self.l = value,
        }
    }

    pub fn get_register16(&self, register: Register16) -> u16 {
        match register {
            Register16::AF => ((self.a as u16) << 8) | self.f as u16,
            Register16::BC => ((self.b as u16) << 8) | self.c as u16,
            Register16::DE => ((self.d as u16) << 8) | self.e as u16,
            Register16::HL => ((self.h as u16) << 8) | self.l as u16,
            Register16::SP => self.sp,
            Register16::PC => self.pc,
        }
    }

    pub fn set_register16(&mut self, register: Register16, value: u16) {
        match register {
            Register16::AF => {
                self.a = (value >> 8) as u8;
                self.f = (value & 0xFF) as u8;
            }
            Register16::BC => {
                self.b = (value >> 8) as u8;
                self.c = (value & 0xFF) as u8;
            }
            Register16::DE => {
                self.d = (value >> 8) as u8;
                self.e = (value & 0xFF) as u8;
            }
            Register16::HL => {
                self.h = (value >> 8) as u8;
                self.l = (value & 0xFF) as u8;
            }
            Register16::SP => self.sp = value,
            Register16::PC => self.pc = value,
        }
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Zero => self.f & 0b10000000 != 0,
            Flag::Negative => self.f & 0b01000000 != 0,
            Flag::HalfCarry => self.f & 0b00100000 != 0,
            Flag::Carry => self.f & 0b00010000 != 0,
        }
    }

    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        if value {
            match flag {
                Flag::Zero => self.f |= 0b10000000,
                Flag::Negative => self.f |= 0b01000000,
                Flag::HalfCarry => self.f |= 0b00100000,
                Flag::Carry => self.f |= 0b00010000,
            }
        } else {
            match flag {
                Flag::Zero => self.f &= 0b01111111,
                Flag::Negative => self.f &= 0b10111111,
                Flag::HalfCarry => self.f &= 0b11011111,
                Flag::Carry => self.f &= 0b11101111,
            }
        }
    }

    pub fn print(&self) {
        println!(
            "a:{:02X}, f:{:02X}, b:{:02X}, c:{:02X}, d:{:02X}, e:{:02X}, h:{:02X}, l:{:02X}, sp:{:04X}, pc:{:04X}",
            self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.sp, self.pc
        );
    }
}
