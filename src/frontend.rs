use raylib::{
    ffi::{GenImageColor, ImageDrawPixel},
    prelude::*,
};

use crate::core::gameboy::CycleInfo;

pub struct Frontend {
    raylib: RaylibHandle,
    thread: RaylibThread,
    framebuffer: Image,
}

impl Frontend {
    pub fn new() -> Self {
        let (raylib, thread) = raylib::init()
            .size(1280, 720)
            .log_level(TraceLogLevel::LOG_NONE)
            .title("Gameboy!")
            .build();

        let framebuffer = unsafe {
            let raw = GenImageColor(160, 144, Color::BLACK);
            Image::from_raw(raw)
        };

        Self {
            raylib,
            thread,
            framebuffer,
        }
    }

    pub fn set_pixels(&mut self, pixels: [u8; 144 * 160]) {
        for y in 0..144 {
            for x in 0..160 {
                let colour = match pixels[x + y * 160] {
                    0 => Color::new(255, 255, 255, 255),
                    1 => Color::new(170, 170, 170, 255),
                    2 => Color::new(85, 85, 85, 255),
                    3 => Color::new(0, 0, 0, 255),
                    _ => unreachable!(),
                };

                self.framebuffer.draw_pixel(x as i32, y as i32, colour);
            }
        }
    }

    pub fn update_debug_info(&mut self, cycle_info: CycleInfo) {
        println!("{:?}", cycle_info);
    }

    pub fn render(&mut self) {
        let texture = self
            .raylib
            .load_texture_from_image(&self.thread, &self.framebuffer)
            .unwrap();

        let mut draw = self.raylib.begin_drawing(&self.thread);
        draw.clear_background(Color::RED);

        draw.draw_texture_ex(&texture, Vector2::new(0.0, 0.0), 0.0, 5.0, Color::WHITE);
    }

    pub fn should_close(&self) -> bool {
        self.raylib.window_should_close()
    }
}
