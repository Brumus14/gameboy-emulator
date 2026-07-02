pub struct Serial {
    data: u8,
    control: u8,
}

impl Serial {
    pub fn new() -> Self {
        Self {
            data: 0,
            control: 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF01 => self.data,
            0xFF02 => self.control,
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF01 => self.data = value,
            0xFF02 => self.control = value,
            _ => unreachable!(),
        }
    }
}
