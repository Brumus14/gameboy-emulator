pub struct Joypad {
    register: u8,
    pub start: bool,
    pub select: bool,
    pub b: bool,
    pub a: bool,
    pub down: bool,
    pub up: bool,
    pub left: bool,
    pub right: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            register: 0,
            start: false,
            select: false,
            b: false,
            a: false,
            down: false,
            up: false,
            left: false,
            right: false,
        }
    }

    pub fn cycle(&mut self) {}

    pub fn read(&self) -> u8 {
        let mut value: u8 = 0;

        if (self.register >> 4) & 1 == 0 {
            if self.right {
                value &= !(1);
            } else {
                value |= 1;
            }

            if self.left {
                value &= !(1 << 1);
            } else {
                value |= 1 << 1;
            }

            if self.up {
                value &= !(1 << 2);
            } else {
                value |= 1 << 2;
            }

            if self.down {
                value &= !(1 << 3);
            } else {
                value |= 1 << 3;
            }
        }

        if (self.register >> 5) & 1 == 0 {
            if self.a {
                value &= !(1);
            } else {
                value |= 1;
            }

            if self.b {
                value &= !(1 << 1);
            } else {
                value |= 1 << 1;
            }

            if self.select {
                value &= !(1 << 2);
            } else {
                value |= 1 << 2;
            }

            if self.start {
                value &= !(1 << 3);
            } else {
                value |= 1 << 3;
            }
        }

        value
    }

    pub fn write(&mut self, value: u8) {
        self.register = (value & 0xF0) | (self.register & 0xF);
    }
}
