pub struct Timer {
    div: u8,
    tima: u8,
    tma: u8,
    tac: u8,
    tick_count: u64,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            tick_count: 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF04 => self.div,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => self.div = 0,
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => self.tac = value,
            _ => unreachable!(),
        }
    }

    // Return true if should timer interrupt
    pub fn cycle(&mut self) -> bool {
        self.tick_count += 1;

        if self.tick_count % 256 == 0 {
            self.div = self.div.wrapping_add(1); // Is this correct?
        }

        let enabled = (self.tac >> 2) & 1 == 1;
        let frequency = match self.tac & 0b00000011 {
            0b00 => 256,
            0b01 => 4,
            0b10 => 16,
            0b11 => 64,
            _ => unreachable!(),
        } * 4;

        if enabled && self.tick_count % frequency == 0 {
            if self.tima == 0xFF {
                self.tima = self.tma;
                return true;
            } else {
                self.tima += 1;
            }
        }

        false
    }
}
