use std::{thread::sleep, time::Duration};

use raylib::{
    ffi::{GenImageColor, ImageDrawPixel},
    prelude::*,
};

use crate::core::{
    debug::Debug,
    gameboy::CycleInfo,
    joypad::JoypadState,
    registers::{Flag, Registers},
};

const STEP_COUNT_INPUT_MAX_LENGTH: usize = 9;

pub enum Command {
    Pause,
    Unpause,
    TogglePause,
    Restart,
    Step(usize),
    UpdateJoypad(JoypadState),
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

    step_count_input: String,

    pause_button: Rectangle,
    restart_button: Rectangle,
    step_button: Rectangle,
    step_count_box: Rectangle,
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

            step_count_input: "".to_string(),

            pause_button: Rectangle::new(0.0, 0.0, 0.0, 0.0),
            restart_button: Rectangle::new(0.0, 0.0, 0.0, 0.0),
            step_button: Rectangle::new(0.0, 0.0, 0.0, 0.0),
            step_count_box: Rectangle::new(0.0, 0.0, 0.0, 0.0),
        };

        frontend.initialise(pixels, next_opcode_bytes, next_opcode_address, registers);

        frontend
    }

    pub fn initialise(
        &mut self,
        pixels: [u8; 144 * 160],
        next_opcode_bytes: [u8; 3],
        next_opcode_address: u16,
        registers: &Registers,
    ) {
        let framebuffer_image = unsafe {
            let mut raw = GenImageColor(160, 144, Color::BLACK);
            raw.format = PixelFormat::PIXELFORMAT_UNCOMPRESSED_GRAYSCALE as i32;
            Image::from_raw(raw)
        };

        let framebuffer = self
            .raylib
            .load_texture_from_image(&self.thread, &framebuffer_image)
            .unwrap();

        self.framebuffer_texture = framebuffer;
        self.framebuffer = [0; 144 * 160];

        self.mouse_left_down = false;

        self.opcode = "".to_string();
        self.next_opcode = "".to_string();

        self.af = "".to_string();
        self.bc = "".to_string();
        self.de = "".to_string();
        self.hl = "".to_string();
        self.sp = "".to_string();
        self.pc = "".to_string();
        self.flags = "".to_string();

        self.step_count_input = "1".to_string();

        let width = self.font_width;
        let height = self.font_height;

        self.pause_button = Rectangle::new(
            800.0 + width,
            720.0 - 4.0 * height,
            7.0 * width,
            3.0 * height,
        );

        self.restart_button = Rectangle::new(
            800.0 + 9.0 * width,
            720.0 - 4.0 * height,
            9.0 * width,
            3.0 * height,
        );

        self.step_button = Rectangle::new(
            800.0 + 1.0 * width,
            720.0 - 8.0 * height,
            6.0 * width,
            3.0 * height,
        );

        self.step_count_box = Rectangle::new(
            800.0 + 8.0 * width,
            720.0 - 8.0 * height,
            11.0 * width,
            3.0 * height,
        );

        self.trace_registers(registers);
        self.set_pixels(pixels);

        let next_opcode = Debug::opcode_to_string(next_opcode_bytes).to_uppercase();
        self.next_opcode = format!("{:04X}: {}", next_opcode_address, next_opcode);
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
        if let Some(cpu_cycle_info) = cycle_info.cpu_cycle_info {
            let opcode = Debug::opcode_to_string(cpu_cycle_info.opcode_bytes).to_uppercase();
            self.opcode = format!("{:04X}: {}", cpu_cycle_info.opcode_address, opcode);

            let next_opcode =
                Debug::opcode_to_string(cpu_cycle_info.next_opcode_bytes).to_uppercase();
            self.next_opcode = format!(
                "{:04X}: {}",
                cpu_cycle_info.next_opcode_address, next_opcode
            );

            self.trace_registers(&cpu_cycle_info.registers);
        }
    }

    pub fn update(&mut self) -> Vec<Command> {
        let mut commands = Vec::new();

        commands.push(Command::UpdateJoypad(JoypadState {
            start: self.raylib.is_key_down(KeyboardKey::KEY_SPACE),
            select: self.raylib.is_key_down(KeyboardKey::KEY_LEFT_SHIFT),
            b: self.raylib.is_key_down(KeyboardKey::KEY_K),
            a: self.raylib.is_key_down(KeyboardKey::KEY_J),
            down: self.raylib.is_key_down(KeyboardKey::KEY_S),
            up: self.raylib.is_key_down(KeyboardKey::KEY_W),
            left: self.raylib.is_key_down(KeyboardKey::KEY_A),
            right: self.raylib.is_key_down(KeyboardKey::KEY_D),
        }));

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
                .restart_button
                .check_collision_point_rec(self.raylib.get_mouse_position())
            {
                commands.push(Command::Restart);
            } else if self
                .step_button
                .check_collision_point_rec(self.raylib.get_mouse_position())
            {
                if let Ok(c) = self.step_count_input.parse::<usize>() {
                    commands.push(Command::Step(c));
                }
            }
        }

        if self
            .step_count_box
            .check_collision_point_rec(self.raylib.get_mouse_position())
        {
            if let Some(c) = self.raylib.get_char_pressed() {
                if c.is_numeric()
                    && self.step_count_input.len() < STEP_COUNT_INPUT_MAX_LENGTH
                    && !(self.step_count_input.len() == 0 && c == '0')
                {
                    self.step_count_input.push(c);
                }
            }

            if self.raylib.is_key_pressed(KeyboardKey::KEY_BACKSPACE)
                || self
                    .raylib
                    .is_key_pressed_repeat(KeyboardKey::KEY_BACKSPACE)
            {
                self.step_count_input.pop();
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
            self.restart_button.x as i32,
            self.restart_button.y as i32,
            self.restart_button.width as i32,
            self.restart_button.height as i32,
            Color::WHITE,
        );

        draw.draw_text_pro(
            &self.font,
            "RESTART",
            Vector2::new(
                self.restart_button.x + self.font_width,
                self.restart_button.y + self.font_height,
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

        draw.draw_rectangle(
            self.step_count_box.x as i32,
            self.step_count_box.y as i32,
            self.step_count_box.width as i32,
            self.step_count_box.height as i32,
            Color::WHITE,
        );

        draw.draw_text_pro(
            &self.font,
            self.step_count_input.as_str(),
            Vector2::new(
                self.step_count_box.x + self.font_width,
                self.step_count_box.y + self.font_height,
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
