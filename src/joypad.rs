pub struct Joypad {
    register: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self { register: 0 }
    }

    pub fn read(&self) -> u8 {
        self.register
    }

    pub fn write(&mut self, value: u8) {
        self.register = value & 0b00110000;
    }
}
