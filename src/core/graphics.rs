pub struct Graphics {
    dot_count: u16,
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
    wy: u8,
    wx: u8,

    x: u8,
    y_condition: bool,
    counter: u8,
    window_active: bool,
    window_row: u8,

    penalty: u16,

    overlapping_object_indexes: Vec<u8>,
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
            y_condition: false,
            counter: 0,
            window_active: false,
            window_row: 0,

            penalty: 0,

            overlapping_object_indexes: Vec::new(),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x8000..0xA000 => {
                // if self.get_mode() != 3 {
                self.video_ram[(address - 0x8000) as usize]
                // } else {
                //     0xFF
                // }
            }
            0xFE00..0xFEA0 => {
                // if self.get_mode() != 2 && self.get_mode() != 3 {
                self.object_attribute_memory[(address - 0xFE00) as usize]
                // } else {
                //     0xFF
                // }
            }
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
            0x8000..0xA000 => {
                // if self.get_mode() != 3 {
                self.video_ram[(address - 0x8000) as usize] = value
                // }
            }
            0xFE00..0xFEA0 => {
                // if self.get_mode() != 2 && self.get_mode() != 3 {
                self.object_attribute_memory[(address - 0xFE00) as usize] = value
                // }
            }
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
        if self.counter == self.wx && (self.lcdc >> 5) & 1 == 1 {
            self.window_row += 1;
            self.window_active = true;
        }

        if self.lcdc & 1 == 0 {
            self.pixels[((self.x as u16) + (self.ly as u16) * 160) as usize] = 0;
            return;
        }

        let tile_y = self.ly.wrapping_add(self.scy) / 8;
        let tile_row = self.ly.wrapping_add(self.scy) % 8;

        let tile_x = self.x.wrapping_add(self.scx) / 8;
        let tile_column = self.x.wrapping_add(self.scx) % 8;

        let tile_map_address_offset = if !self.window_active {
            if (self.lcdc >> 3) & 1 == 1 {
                0x1C00
            } else {
                0x1800
            }
        } else {
            if (self.lcdc >> 6) & 1 == 1 {
                0x1C00
            } else {
                0x1800
            }
        };

        let tile_index = self.video_ram
            [(tile_map_address_offset + (tile_x as u16) + (tile_y as u16) * 32) as usize];

        let tile_address_offset = if (self.lcdc >> 4) & 1 == 1 {
            (tile_index as u16) * 16
        } else {
            0x1000u16.wrapping_add_signed((tile_index as i8 as i16) * 16)
        };

        let tile_row_address_offset = tile_address_offset + (tile_row as u16) * 2;

        let low = (self.video_ram[tile_row_address_offset as usize] >> (8 - tile_column - 1)) & 1;
        let high =
            (self.video_ram[(tile_row_address_offset + 1) as usize] >> (8 - tile_column - 1)) & 1;
        let palette_index = (high << 1) | low;

        let mut value = (self.bgp >> (palette_index * 2)) & 0b11;

        for index in &self.overlapping_object_indexes {
            let x = self.object_attribute_memory[(*index as usize) * 4 + 1];

            if self.x + 8 >= x && self.x < x {
                let y = self.object_attribute_memory[(*index as usize) * 4];
                let tile_index = self.object_attribute_memory[(*index as usize) * 4 + 2];
                let flags = self.object_attribute_memory[(*index as usize) * 4 + 3];

                let tile_address_offset = (tile_index as u16) * 16;

                let tile_row = if (flags >> 6) & 1 == 1 {
                    7 - ((self.ly + 16 - y) % 8)
                } else {
                    (self.ly + 16 - y) % 8
                };

                let tile_column = if (flags >> 5) & 1 == 1 {
                    7 - ((self.x + 8 - x) % 8)
                } else {
                    (self.x + 8 - x) % 8
                };

                let tile_row_address_offset = tile_address_offset + (tile_row as u16) * 2;

                let low =
                    (self.video_ram[tile_row_address_offset as usize] >> (8 - tile_column - 1)) & 1;
                let high = (self.video_ram[(tile_row_address_offset + 1) as usize]
                    >> (8 - tile_column - 1))
                    & 1;
                let object_value = (high << 1) | low;

                if object_value != 0 {
                    value = object_value;
                }
            }
        }

        self.pixels[((self.x as u16) + (self.ly as u16) * 160) as usize] = value;
    }

    // Return true if should vblank interrupt
    pub fn cycle(&mut self) -> bool {
        let mut interrupt = false;

        if self.dot_count == 0 {
            self.counter = 0;

            if self.wy == self.ly {
                self.y_condition = true;
            }

            self.window_active = false;
        }

        if self.ly < 144 {
            if self.dot_count == 0 {
                self.set_mode(2);
                self.penalty = 0;

                let object_pixel_height = if (self.lcdc >> 2) & 1 == 1 { 16 } else { 8 };
                self.overlapping_object_indexes.clear();

                for i in 0..40 {
                    let y = self.object_attribute_memory[i * 4];

                    if self.ly + 16 >= y && self.ly + 16 < y + object_pixel_height {
                        self.overlapping_object_indexes.push(i as u8);
                    }
                }
            } else if self.dot_count == 80 {
                self.counter += 7;
                self.set_mode(3);
                self.penalty += 12 + (self.scx % 8) as u16;
            }

            if self.get_mode() == 3 && self.penalty == 0 {
                self.draw_pixel();

                if self.x == 159 {
                    self.set_mode(0);
                } else {
                    self.x += 1;
                }
            }
        } else if self.ly == 144 && self.dot_count == 0 {
            self.set_mode(1);
            self.y_condition = false;
        }

        self.dot_count += 1;

        if self.penalty > 0 {
            self.penalty -= 1;
        }

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

    pub fn stat(&self) -> u8 {
        self.stat
    }

    pub fn lcdc(&self) -> u8 {
        self.lcdc
    }
}
