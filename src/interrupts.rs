pub struct Interrupts {
    pub enabled: u8,
    flags: u8,
}

impl Interrupts {
    pub fn new() -> Self {
        Self {
            enabled: 0,
            flags: 0,
        }
    }

    pub fn read(&self) -> u8 {
        0
    }

    pub fn write(&mut self, value: u8) {}

    pub fn handle(&mut self) {
        for i in 0..5 {
            let enabled = (self.enabled >> i) & 1 == 1;
            let requested = (self.flags >> i) & 1 == 1;

            if enabled && requested {}
        }
    }
}
