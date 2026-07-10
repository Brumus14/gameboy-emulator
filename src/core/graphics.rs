pub struct Graphics {
    dot_count: usize,
    pixels: [u8; 144 * 160],
    video_ram: [u8; 8192],
    object_attribute_memory: [u8; 160],
    lcdc: u8,
    pub stat: u8,
    scy: u8,
    scx: u8,
    ly: u8,
    lyc: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
}

impl Graphics {
    pub fn new() -> Self {
        Self {
            dot_count: 0,
            pixels: [0; 144 * 160],
            video_ram: [0; 8192],
            object_attribute_memory: [0; 160],
            lcdc: 0,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x8000..0xA000 => self.video_ram[(address - 0x8000) as usize],
            0xFE00..0xFEA0 => self.object_attribute_memory[(address - 0xFE00) as usize],
            0xFF40 => self.lcdc,
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF46 => todo!(),
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => todo!(),
            0xFF4B => todo!(),
            _ => unreachable!(),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x8000..0xA000 => self.video_ram[(address - 0x8000) as usize] = value,
            0xFE00..0xFEA0 => self.object_attribute_memory[(address - 0xFE00) as usize] = value,
            0xFF40 => self.lcdc = value,
            0xFF41 => self.stat = value & 0b11111100,
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => (),
            0xFF45 => self.lyc = value,
            0xFF46 => todo!(),
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => (),
            0xFF4B => (),
            _ => unreachable!(),
        }
    }

    pub fn cycle(&mut self) {
        self.dot_count += 1;

        if self.dot_count < 456 {
            return;
        } else {
            self.dot_count -= 456;
        }

        let bg_address = if self.lcdc & 0b00001000 == 0 {
            0x9800
        } else {
            0x9C00
        };

        let line = self.ly;

        if line >= 144 {
            self.stat = (self.stat & 0b11111100) | 0b00000001;
            self.ly = (line + 1) % 154;
            return;
        } else {
            self.stat = (self.stat & 0b11111100) | 0b00000011;
        }

        // let tile_y = line.wrapping_add(self.scy) / 8;
        // let tile_row = line.wrapping_add(self.scy) % 8;
        let tile_y = line / 8;
        let tile_row = line % 8;

        for x in 0..160u8 {
            let tile_x = x.wrapping_add(self.scx) / 8;
            let tile_column = x.wrapping_add(self.scx) % 8;

            let index = self.read(bg_address + (tile_x as u16) + (tile_y as u16) * 32);

            let tile_address = if (self.lcdc >> 4) & 1 == 1 {
                0x8000 + (index as u16) * 16
            } else {
                0x9000u16.wrapping_add_signed((index as i8 as i16) * 16)
            };
            let tile_row_address = tile_address + (tile_row as u16) * 2;

            let right = (self.read(tile_row_address) >> (8 - tile_column - 1)) & 1;
            let left = (self.read(tile_row_address + 1) >> (8 - tile_column - 1)) & 1;

            self.pixels[((x as u16) + (line as u16) * 160) as usize] = (left << 1) | right;
        }

        self.ly = (line + 1) % 154;
    }

    pub fn pixels(&self) -> [u8; 144 * 160] {
        self.pixels
    }
}
