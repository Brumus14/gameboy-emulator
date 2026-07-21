use std::time::Instant;

pub struct Graphics {
    dot_count: u16,
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
    wy: u8,
    wx: u8,
    x: u8,
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
            wy: 0,
            wx: 0,
            x: 0,
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
            0xFF46 => 0, // TODO: Implement
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
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
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            _ => unreachable!(),
        }
    }

    pub fn get_mode(&self) -> u8 {
        self.stat & 0b00000011
    }

    fn set_mode(&mut self, mode: u8) {
        self.stat = (self.stat & 0b11111100) | (mode & 0b00000011);
    }

    fn draw_pixel(&mut self) {
        if self.lcdc & 1 == 0 {
            self.pixels[((self.x as u16) + (self.ly as u16) * 160) as usize] = 0;
            return;
        }

        let tile_y = self.ly.wrapping_add(self.scy) / 8;
        let tile_row = self.ly.wrapping_add(self.scy) % 8;

        let tile_x = self.x.wrapping_add(self.scx) / 8;
        let tile_column = self.x.wrapping_add(self.scx) % 8;

        let bg_address = if (self.lcdc >> 3) & 1 == 1 {
            0x9C00
        } else {
            0x9800
        };

        let index = self.read(bg_address + (tile_x as u16) + (tile_y as u16) * 32);

        let tile_address = if (self.lcdc >> 4) & 1 == 1 {
            0x8000 + (index as u16) * 16
        } else {
            0x9000u16.wrapping_add_signed((index as i8 as i16) * 16)
        };
        let tile_row_address = tile_address + (tile_row as u16) * 2;

        let low = (self.read(tile_row_address) >> (8 - tile_column - 1)) & 1;
        let high = (self.read(tile_row_address + 1) >> (8 - tile_column - 1)) & 1;
        let palette_index = (high << 1) | low;

        let value = (self.read(0xFF47) >> (palette_index * 2)) & 0b11;

        self.pixels[((self.x as u16) + (self.ly as u16) * 160) as usize] = value;
    }

    // Return true if should timer interrupt
    pub fn cycle(&mut self) -> bool {
        let mut interrupt = false;
        let scroll_penalty = (self.scx % 8) as u16;

        if self.ly < 144 {
            if self.dot_count == 0 {
                self.set_mode(2);
            } else if self.dot_count == 80 {
                self.set_mode(3);
            } else if self.dot_count >= 92 + scroll_penalty && self.get_mode() != 0 {
                self.draw_pixel();

                if self.x == 159 {
                    self.set_mode(0);
                } else {
                    self.x += 1;
                }
            }
        } else if self.ly == 144 && self.dot_count == 0 {
            self.set_mode(1);
        }

        self.dot_count += 1;

        if self.dot_count == 456 {
            self.dot_count -= 456;
            self.ly = (self.ly + 1) % 154;
            self.x = 0;

            if self.ly == 144 {
                interrupt = true;
            }
        }

        if self.lyc == self.ly {
            self.stat |= 0b00000100;
        } else {
            self.stat &= 0b11111011;
        }

        interrupt
    }

    pub fn pixels(&self) -> [u8; 144 * 160] {
        self.pixels
    }
}
