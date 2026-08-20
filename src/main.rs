use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{
    core::{
        cartridge::Cartridge,
        debug::Debug,
        gameboy::{CycleInfo, Gameboy},
        timer::Timer,
    },
    frontend::Frontend,
};

mod core;
mod frontend;

const FRAME_DURATION: Duration = Duration::from_micros(16740);

enum CycleState {
    Unpaused,
    Paused,
    Restarted,
    Step(usize),
}

fn main() {
    let mut gameboy = Gameboy::new();

    let cartridge = Cartridge::from_file("./res/rom/KirbysDreamLand.gb").unwrap();
    // let cartridge =
    //     Cartridge::from_file("./res/rom/blarggtests/interrupt_time/interrupt_time.gb").unwrap();
    gameboy.load_cartridge(cartridge);

    let mut cycle_state = CycleState::Paused;

    let (next_opcode_bytes, next_opcode_address) = gameboy.get_next_opcode();
    let mut frontend = Frontend::new(
        gameboy.get_pixels(),
        next_opcode_bytes,
        next_opcode_address,
        &gameboy.get_registers(),
    );

    let mut frame_rendered = false;
    let mut frame_render_time = Instant::now();

    while !frontend.should_close() {
        let commands = frontend.update();

        for command in commands {
            match command {
                frontend::Command::UpdateJoypad(state) => {
                    gameboy.update_joypad_state(state);
                }
                frontend::Command::Pause => cycle_state = CycleState::Paused,
                frontend::Command::Unpause => cycle_state = CycleState::Unpaused,
                frontend::Command::TogglePause => {
                    if let CycleState::Paused = cycle_state {
                        cycle_state = CycleState::Unpaused;
                    } else {
                        cycle_state = CycleState::Paused;
                    }
                }
                frontend::Command::Restart => {
                    gameboy.restart();
                    cycle_state = CycleState::Restarted;
                }
                frontend::Command::Step(s) => match cycle_state {
                    CycleState::Paused => cycle_state = CycleState::Step(s),
                    CycleState::Step(current_steps) => {
                        cycle_state = CycleState::Step(current_steps + s)
                    }
                    _ => (),
                },
            }
        }

        match cycle_state {
            CycleState::Unpaused => {
                let mut cycle_info: Option<CycleInfo> = None;

                while !gameboy.frame_ready() || frame_rendered {
                    if !gameboy.frame_ready() && frame_rendered {
                        frame_rendered = false;
                    }

                    cycle_info = Some(gameboy.cycle());

                    // let opcode = gameboy.get_next_opcode();
                    //
                    // if gameboy.get_next_opcode().1 & 0xF000 >= 0x8000 {
                    // if opcode.1 == 0xCE65 {
                    // if gameboy.get_next_opcode().0[0] == 0b00000001
                    //     && gameboy.get_next_opcode().0[1] == 0
                    //     && gameboy.get_next_opcode().0[2] == 0x12 {
                    //     cycle_state = CycleState::Paused;
                    //     break;
                    // }
                }

                if let Some(cycle_info) = cycle_info {
                    frontend.trace_cycle_info(cycle_info);
                    frontend.set_pixels(gameboy.get_pixels());

                    let duration = Instant::now() - frame_render_time;

                    if duration < FRAME_DURATION {
                        sleep(FRAME_DURATION - duration);
                    }

                    frame_rendered = true;
                    frame_render_time = Instant::now();
                }
            }
            CycleState::Restarted => {
                let (next_opcode_bytes, next_opcode_address) = gameboy.get_next_opcode();
                frontend.initialise(
                    gameboy.get_pixels(),
                    next_opcode_bytes,
                    next_opcode_address,
                    &gameboy.get_registers(),
                );

                cycle_state = CycleState::Paused;
            }
            CycleState::Step(ref mut remaining_steps) => {
                let mut cycle_info: Option<CycleInfo> = None;

                while *remaining_steps > 0 && (!gameboy.frame_ready() || frame_rendered) {
                    if !gameboy.frame_ready() && frame_rendered {
                        frame_rendered = false;
                    }

                    cycle_info = Some(gameboy.cycle());
                    *remaining_steps -= 1;
                }

                if let Some(cycle_info) = cycle_info {
                    frontend.trace_cycle_info(cycle_info);
                    frontend.set_pixels(gameboy.get_pixels());

                    frame_rendered = true;
                }

                if *remaining_steps == 0 {
                    cycle_state = CycleState::Paused;
                }
            }
            _ => (),
        }

        frontend.render();
    }
}
