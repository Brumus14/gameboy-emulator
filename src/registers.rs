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

pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

pub enum Flag {
    Zero,
    Subtraction,
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
            sp: 0,
            pc: 0,
        }
    }

    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }

    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }

    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Zero => self.f & 0b10000000 != 0,
            Flag::Subtraction => self.f & 0b01000000 != 0,
            Flag::HalfCarry => self.f & 0b00100000 != 0,
            Flag::Carry => self.f & 0b00010000 != 0,
        }
    }

    pub fn zero(&self) -> bool {
        (self.f >> ZERO_FLAG_BIT) & 1 == 1
    }

    pub fn subtraction(&self) -> bool {
        (self.f >> SUBTRACTION_FLAG_BIT) & 1 == 1
    }

    pub fn half_carry(&self) -> bool {
        (self.f >> HALF_CARRY_FLAG_BIT) & 1 == 1
    }

    pub fn carry(&self) -> bool {
        (self.f >> CARRY_FLAG_BIT) & 1 == 1
    }
}
