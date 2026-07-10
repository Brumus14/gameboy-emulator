use std::{thread::sleep, time::Duration};

use raylib::{
    ffi::{GenImageColor, ImageDrawPixel},
    prelude::*,
};

use crate::core::{
    debug::Debug,
    gameboy::CycleInfo,
    registers::{Flag, Registers},
};

pub enum Command {
    Pause,
    Unpause,
    TogglePause,
    Step(usize),
}

pub struct Frontend {
    raylib: RaylibHandle,
    thread: RaylibThread,
    font: Font,
    font_width: f32,
    font_height: f32,
    framebuffer_texture: Texture2D,
    framebuffer: [u8; 144 * 160],

    mouse_left_down: bool,

    opcode: String,
    next_opcode: String,

    af: String,
    bc: String,
    de: String,
    hl: String,
    sp: String,
    pc: String,
    flags: String,

    pause_button: Rectangle,
    step_button: Rectangle,
}

impl Frontend {
    pub fn new(
        pixels: [u8; 144 * 160],
        next_opcode_bytes: [u8; 3],
        next_opcode_address: u16,
        registers: &Registers,
    ) -> Self {
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

        let framebuffer_image = unsafe {
            let mut raw = GenImageColor(160, 144, Color::BLACK);
            raw.format = PixelFormat::PIXELFORMAT_UNCOMPRESSED_GRAYSCALE as i32;
            Image::from_raw(raw)
        };

        let framebuffer = raylib
            .load_texture_from_image(&thread, &framebuffer_image)
            .unwrap();

        let Vector2 {
            x: font_width,
            y: font_height,
        } = font.measure_text(" ", 24.0, 0.0);

        let mut frontend = Self {
            raylib,
            thread,
            font,
            font_width,
            font_height,
            framebuffer_texture: framebuffer,
            framebuffer: [0; 144 * 160],

            mouse_left_down: false,

            opcode: "".to_string(),
            next_opcode: "".to_string(),

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

            step_button: Rectangle::new(
                800.0 + 9.0 * width,
                720.0 - 4.0 * height,
                6.0 * width,
                3.0 * height,
            ),
        };

        frontend.trace_registers(registers);
        frontend.set_pixels(pixels);

        let next_opcode = Debug::opcode_to_string(next_opcode_bytes).to_uppercase();
        frontend.next_opcode = format!("{:04X}: {}", next_opcode_address, next_opcode);

        frontend
    }

    pub fn set_pixels(&mut self, pixels: [u8; 144 * 160]) {
        for y in 0..144 {
            for x in 0..160 {
                let colour = match pixels[x + y * 160] {
                    0 => 255,
                    1 => 170,
                    2 => 85,
                    3 => 0,
                    _ => unreachable!(),
                };

                self.framebuffer[y * 160 + x] = colour;
            }
        }

        self.framebuffer_texture
            .update_texture(&self.framebuffer)
            .unwrap();
    }

    pub fn trace_registers(&mut self, registers: &Registers) {
        self.af = format!("AF: {:02X} {:02X}", registers.a, registers.f);
        self.bc = format!("BC: {:02X} {:02X}", registers.b, registers.c);
        self.de = format!("DE: {:02X} {:02X}", registers.d, registers.e);
        self.hl = format!("HL: {:02X} {:02X}", registers.h, registers.l);
        self.sp = format!("SP: {:04X}", registers.sp);
        self.pc = format!("PC: {:04X}", registers.pc);

        self.flags = format!("ZNHC: {:04b}", registers.f >> 4);
    }

    pub fn trace_cycle_info(&mut self, cycle_info: CycleInfo) {
        let cpu_cycle_info = cycle_info.cpu_cycle_info;

        let opcode = Debug::opcode_to_string(cpu_cycle_info.opcode_bytes).to_uppercase();
        self.opcode = format!("{:04X}: {}", cpu_cycle_info.opcode_address, opcode);

        let next_opcode = Debug::opcode_to_string(cpu_cycle_info.next_opcode_bytes).to_uppercase();
        self.next_opcode = format!(
            "{:04X}: {}",
            cpu_cycle_info.next_opcode_address, next_opcode
        );

        self.trace_registers(&cpu_cycle_info.registers);
    }

    pub fn update(&mut self) -> Vec<Command> {
        let mut commands = Vec::new();

        let mouse_left_down = self
            .raylib
            .is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);

        let mouse_left_pressed = !self.mouse_left_down && mouse_left_down;

        if mouse_left_pressed {
            if self
                .pause_button
                .check_collision_point_rec(self.raylib.get_mouse_position())
            {
                commands.push(Command::TogglePause);
            } else if self
                .step_button
                .check_collision_point_rec(self.raylib.get_mouse_position())
            {
                commands.push(Command::Step(1));
            }
        }

        self.mouse_left_down = mouse_left_down;

        commands
    }

    pub fn render(&mut self) {
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
            Vector2::new(
                self.pause_button.x + self.font_width,
                self.pause_button.y + self.font_height,
            ),
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
            Vector2::new(
                self.step_button.x + self.font_width,
                self.step_button.y + self.font_height,
            ),
            Vector2::ZERO,
            0.0,
            24.0,
            0.0,
            Color::BLACK,
        );

        draw.draw_text_ex(
            &self.font,
            &self.opcode,
            Vector2::new(800.0 + self.font_width, self.font_height),
            24.0,
            0.0,
            Color::WHITE,
        );
        draw.draw_text_ex(
            &self.font,
            &self.next_opcode,
            Vector2::new(800.0 + self.font_width, 2.0 * self.font_height),
            24.0,
            0.0,
            Color::WHITE,
        );

        draw.draw_text_ex(
            &self.font,
            &self.af,
            Vector2::new(800.0 + self.font_width, 3.0 * self.font_height),
            24.0,
            0.0,
            Color::WHITE,
        );
        draw.draw_text_ex(
            &self.font,
            &self.bc,
            Vector2::new(800.0 + self.font_width, 4.0 * self.font_height),
            24.0,
            0.0,
            Color::WHITE,
        );
        draw.draw_text_ex(
            &self.font,
            &self.de,
            Vector2::new(800.0 + self.font_width, 5.0 * self.font_height),
            24.0,
            0.0,
            Color::WHITE,
        );
        draw.draw_text_ex(
            &self.font,
            &self.hl,
            Vector2::new(800.0 + self.font_width, 6.0 * self.font_height),
            24.0,
            0.0,
            Color::WHITE,
        );
        draw.draw_text_ex(
            &self.font,
            &self.sp,
            Vector2::new(800.0 + self.font_width, 7.0 * self.font_height),
            24.0,
            0.0,
            Color::WHITE,
        );
        draw.draw_text_ex(
            &self.font,
            &self.pc,
            Vector2::new(800.0 + self.font_width, 8.0 * self.font_height),
            24.0,
            0.0,
            Color::WHITE,
        );

        draw.draw_text_ex(
            &self.font,
            &self.flags,
            Vector2::new(800.0 + self.font_width, 9.0 * self.font_height),
            24.0,
            0.0,
            Color::WHITE,
        );

        draw.draw_texture_ex(
            &self.framebuffer_texture,
            Vector2::new(0.0, 0.0),
            0.0,
            5.0,
            Color::WHITE,
        );
    }

    pub fn should_close(&self) -> bool {
        self.raylib.window_should_close()
    }
}
