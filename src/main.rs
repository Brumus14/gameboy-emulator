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

enum CycleState {
    Unpaused,
    Paused,
    Step(usize),
}

fn main() {
    let mut gameboy = Gameboy::new();

    let cartridge = Cartridge::from_file("./res/rom/Tetris.gb").unwrap();
    gameboy.load_cartridge(cartridge);

    let mut cycle_state = CycleState::Paused;

    let (opcode_bytes, opcode_address) = gameboy.get_next_opcode();
    let mut frontend = Frontend::new(
        gameboy.get_pixels(),
        opcode_bytes,
        opcode_address,
        &gameboy.get_registers(),
    );

    while !frontend.should_close() {
        let commands = frontend.update();

        for command in commands {
            match command {
                frontend::Command::Pause => cycle_state = CycleState::Paused,
                frontend::Command::Unpause => cycle_state = CycleState::Unpaused,
                frontend::Command::TogglePause => {
                    if let CycleState::Paused = cycle_state {
                        cycle_state = CycleState::Unpaused;
                    } else {
                        cycle_state = CycleState::Paused;
                    }
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

                while !gameboy.frame_ready() {
                    cycle_info = Some(gameboy.cycle());

                    // if gameboy.get_next_opcode().1 == 0x40 {
                    //     cycle_state = CycleState::Paused;
                    //     break;
                    // }
                }

                if let Some(cycle_info) = cycle_info {
                    frontend.trace_cycle_info(cycle_info);
                    frontend.set_pixels(gameboy.get_pixels());
                }
            }
            CycleState::Step(ref mut remaining_steps) => {
                let cycle_info = gameboy.cycle();

                *remaining_steps -= 1;

                if *remaining_steps == 0 {
                    cycle_state = CycleState::Paused;
                }

                frontend.trace_cycle_info(cycle_info);
                frontend.set_pixels(gameboy.get_pixels());
            }
            _ => (),
        }

        frontend.render();
    }
}
