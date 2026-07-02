pub struct Ppu {
    video_ram: [u8; 8192],
    object_attribute_memory: [u8; 160],
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            video_ram: [0; 8192],
            object_attribute_memory: [0; 160],
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        0
    }

    pub fn write(&mut self, address: u16, value: u8) {}

    pub fn render(&mut self) {}
}
