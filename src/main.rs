use std::{thread::sleep, time::Duration};

use crate::{
    core::{
        cartridge::Cartridge,
        debug::Debug,
        gameboy::{CycleInfo, Gameboy},
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

    let mut frontend = Frontend::new();

    while !frontend.should_close() {
        let commands = frontend.update();

        for command in commands {
            match command {
                frontend::Command::Pause => cycle_state = CycleState::Paused,
                frontend::Command::Unpause => cycle_state = CycleState::Unpaused,
                frontend::Command::Step(s) => {
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

        if let CycleState::Unpaused = cycle_state {
            let cycle_info = gameboy.cycle();
            frontend.update_debug_info(cycle_info);
            frontend.set_pixels(gameboy.get_pixels());
        } else if let CycleState::Step(s) = cycle_state {
            let cycle_info = gameboy.cycle();
            frontend.update_debug_info(cycle_info);
            frontend.set_pixels(gameboy.get_pixels());

            if s == 1 {
                cycle_state = CycleState::Paused;
            } else {
                cycle_state = CycleState::Step(s - 1);
            }
        }

        frontend.render();
    }
}
