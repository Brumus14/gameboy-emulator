use std::{thread::sleep, time::Duration};

use raylib::{
    ffi::{GenImageColor, ImageDrawPixel},
    prelude::*,
};

use crate::core::{debug::Debug, gameboy::CycleInfo, registers::Flag};

pub enum Command {
    Pause,
    Unpause,
    Step(usize),
}

pub struct Frontend {
    raylib: RaylibHandle,
    thread: RaylibThread,
    font: Font,
    framebuffer: Image,
    opcode: String,
    af: String,
    bc: String,
    de: String,
    hl: String,
    sp: String,
    pc: String,
    flags: String,

    pause_button: Rectangle,
    paused: bool,

    step_button: Rectangle,
}

impl Frontend {
    pub fn new() -> Self {
        let (mut raylib, thread) = raylib::init()
            .size(1280, 720)
            .log_level(TraceLogLevel::LOG_NONE)
            .title("Gameboy!")
            .build();

        let font = raylib
            .load_font(&thread, "res/font/PublicPixel.ttf")
            .unwrap();

        let Vector2 {
            x: width,
            y: height,
        } = font.measure_text(" ", 24.0, 0.0);

        let framebuffer = unsafe {
            let raw = GenImageColor(160, 144, Color::BLACK);
            Image::from_raw(raw)
        };

        Self {
            raylib,
            thread,
            font,
            framebuffer,
            opcode: "".to_string(),
            af: "".to_string(),
            bc: "".to_string(),
            de: "".to_string(),
            hl: "".to_string(),
            sp: "".to_string(),
            pc: "".to_string(),
            flags: "".to_string(),

            pause_button: Rectangle::new(
                800.0 + width,
                720.0 - 4.0 * height,
                7.0 * width,
                3.0 * height,
            ),
            paused: false,

            step_button: Rectangle::new(
                800.0 + 9.0 * width,
                720.0 - 4.0 * height,
                6.0 * width,
                3.0 * height,
            ),
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
        let cpu_cycle_info = cycle_info.cpu_cycle_info;

        self.opcode = Debug::get_opcode_string(cpu_cycle_info).to_uppercase();
        self.af = format!(
            "AF: {:02X} {:02X}",
            cpu_cycle_info.registers.a, cpu_cycle_info.registers.f
        );
        self.bc = format!(
            "BC: {:02X} {:02X}",
            cpu_cycle_info.registers.b, cpu_cycle_info.registers.c
        );
        self.de = format!(
            "DE: {:02X} {:02X}",
            cpu_cycle_info.registers.d, cpu_cycle_info.registers.e
        );
        self.hl = format!(
            "HL: {:02X} {:02X}",
            cpu_cycle_info.registers.h, cpu_cycle_info.registers.l
        );
        self.sp = format!("SP: {:04X}", cpu_cycle_info.registers.sp);
        self.pc = format!("PC: {:04X}", cpu_cycle_info.registers.pc);

        self.flags = format!("ZNHC: {:04b}", cpu_cycle_info.registers.f >> 4);
    }

    pub fn update(&mut self) -> Vec<Command> {
        let mut commands = Vec::new();

        if self
            .raylib
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
        {
            if self
                .pause_button
                .check_collision_point_rec(self.raylib.get_mouse_position())
            {
                self.paused = !self.paused;

                if self.paused {
                    commands.push(Command::Pause);
                } else {
                    commands.push(Command::Unpause);
                }
            } else if self
                .step_button
                .check_collision_point_rec(self.raylib.get_mouse_position())
            {
                commands.push(Command::Step(1));
            }
        }

        commands
    }

    pub fn render(&mut self) {
        let texture = self
            .raylib
            .load_texture_from_image(&self.thread, &self.framebuffer)
            .unwrap();

        let Vector2 {
            x: width,
            y: height,
        } = self.font.measure_text(" ", 24.0, 0.0);

        let mut draw = self.raylib.begin_drawing(&self.thread);
        draw.clear_background(Color::BLACK);

        draw.draw_rectangle(
            self.pause_button.x as i32,
            self.pause_button.y as i32,
            self.pause_button.width as i32,
            self.pause_button.height as i32,
            Color::WHITE,
        );

        draw.draw_text_pro(
            &self.font,
            "PAUSE",
            Vector2::new(self.pause_button.x + width, self.pause_button.y + height),
            Vector2::ZERO,
            0.0,
            24.0,
            0.0,
            Color::BLACK,
        );

        draw.draw_rectangle(
            self.step_button.x as i32,
            self.step_button.y as i32,
            self.step_button.width as i32,
            self.step_button.height as i32,
            Color::WHITE,
        );

        draw.draw_text_pro(
            &self.font,
            "STEP",
            Vector2::new(self.step_button.x + width, self.step_button.y + height),
            Vector2::ZERO,
            0.0,
            24.0,
            0.0,
            Color::BLACK,
        );

        draw.draw_text_ex(
            &self.font,
            &self.opcode,
            Vector2::new(800.0 + width, height),
            24.0,
            0.0,
            Color::WHITE,
        );
        draw.draw_text_ex(
            &self.font,
            &self.af,
            Vector2::new(800.0 + width, 2.0 * height),
            24.0,
            0.0,
            Color::WHITE,
        );
        draw.draw_text_ex(
            &self.font,
            &self.bc,
            Vector2::new(800.0 + width, 3.0 * height),
            24.0,
            0.0,
            Color::WHITE,
        );
        draw.draw_text_ex(
            &self.font,
            &self.de,
            Vector2::new(800.0 + width, 4.0 * height),
            24.0,
            0.0,
            Color::WHITE,
        );
        draw.draw_text_ex(
            &self.font,
            &self.hl,
            Vector2::new(800.0 + width, 5.0 * height),
            24.0,
            0.0,
            Color::WHITE,
        );
        draw.draw_text_ex(
            &self.font,
            &self.sp,
            Vector2::new(800.0 + width, 6.0 * height),
            24.0,
            0.0,
            Color::WHITE,
        );
        draw.draw_text_ex(
            &self.font,
            &self.pc,
            Vector2::new(800.0 + width, 7.0 * height),
            24.0,
            0.0,
            Color::WHITE,
        );

        draw.draw_text_ex(
            &self.font,
            &self.flags,
            Vector2::new(800.0 + width, 8.0 * height),
            24.0,
            0.0,
            Color::WHITE,
        );

        draw.draw_texture_ex(&texture, Vector2::new(0.0, 0.0), 0.0, 5.0, Color::WHITE);

        sleep(Duration::from_millis(100));
    }

    pub fn should_close(&self) -> bool {
        self.raylib.window_should_close()
    }
}
