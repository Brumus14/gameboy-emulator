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

    let mut cycle_state = CycleState::Unpaused;

    let (opcode_bytes, opcode_address) = gameboy.get_next_opcode();
    let mut frontend = Frontend::new(
        gameboy.get_pixels(),
        &gameboy.get_registers(),
        opcode_bytes,
        opcode_address,
    );

    let mut frontend_rendered = false;

    while !frontend.should_close() {
        let commands = frontend.update();

        for command in commands {
            match command {
                frontend::Command::Pause => cycle_state = CycleState::Paused,
                frontend::Command::Unpause => cycle_state = CycleState::Unpaused,
                frontend::Command::Step(s) => {
                    if let CycleState::Paused = cycle_state {
                        // Should this be here?
                        let steps = if let CycleState::Step(current_steps) = cycle_state {
                            current_steps + s
                        } else {
                            s
                        };

                        cycle_state = CycleState::Step(steps);
                    }
                }
            }
        }

        if let CycleState::Unpaused = cycle_state {
            let cycle_info = gameboy.cycle();

            if gameboy.frame_ready() {
                if !frontend_rendered {
                    frontend_rendered = true;
                    frontend.trace_cycle_info(cycle_info);
                    frontend.set_pixels(gameboy.get_pixels());
                    frontend.render();
                }
            } else if frontend_rendered {
                frontend_rendered = false;
            }
        } else if let CycleState::Step(s) = cycle_state {
            let cycle_info = gameboy.cycle();
            frontend.trace_cycle_info(cycle_info);
            frontend.set_pixels(gameboy.get_pixels());

            if s == 1 {
                cycle_state = CycleState::Paused;
            } else {
                cycle_state = CycleState::Step(s - 1);
            }
        }
    }
}
