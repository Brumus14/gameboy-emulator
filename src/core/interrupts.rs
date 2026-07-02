pub struct Interrupts {
    pub enable: u8,
    flag: u8,
}

impl Interrupts {
    pub fn new() -> Self {
        Self { enable: 0, flag: 0 }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFFFF => self.enable,
            0xFF0F => self.flag,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFFFF => self.enable = value,
            0xFF0F => self.flag = value,
            _ => unreachable!(),
        }
    }

    pub fn handle(&mut self) {
        for i in 0..5 {
            let enabled = (self.enable >> i) & 1 == 1;
            let requested = (self.flag >> i) & 1 == 1;

            if enabled && requested {}
        }
    }
}
