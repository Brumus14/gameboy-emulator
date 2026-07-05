pub struct Graphics {
    pixels: [u8; 144 * 160],
    video_ram: [u8; 8192],
    object_attribute_memory: [u8; 160],
    lcdc: u8,
    stat: u8,
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
        println!("{}", address);
        match address {
            0x8000..0xA000 => self.video_ram[(address - 0x8000) as usize] = value,
            0xFE00..0xFEA0 => self.object_attribute_memory[(address - 0xFE00) as usize] = value,
            0xFF40 => self.lcdc = value,
            0xFF41 => self.stat = value & 0b11111100,
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF45 => self.lyc = value,
            0xFF46 => todo!(),
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => todo!(),
            0xFF4B => todo!(),
            _ => unreachable!(),
        }
    }

    pub fn cycle(&mut self) {
        let bg_address = if self.lcdc & 0b00001000 == 0 {
            0x9800
        } else {
            0x9C00
        };

        for y in 0..144 {
            let tile_y = y / 8;
            let tile_row = y % 8;

            for x in 0..160 {
                let tile_x = x / 8;
                let tile_column = x % 8;

                let index = self.read(bg_address + tile_x + tile_y * 32);
                let left = self.read(0x8000 + (index as u16) * 16 + tile_row * 2);
                let right = self.read(0x8000 + (index as u16) * 16 + tile_row * 2 + 1);

                self.pixels[(x + y * 160) as usize] = (((left >> (8 - tile_column - 1)) & 1) << 1)
                    | ((right >> (8 - tile_column - 1)) & 1);
            }
        }
    }

    pub fn get_pixels(&self) -> [u8; 144 * 160] {
        self.pixels
    }
}
