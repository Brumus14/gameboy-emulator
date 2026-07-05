use std::{thread::sleep, time::Duration};

use crate::{
    core::{cartridge::Cartridge, gameboy::Gameboy},
    frontend::Frontend,
};

mod core;
mod frontend;

fn main() {
    let mut gameboy = Gameboy::new();
    let cartridge = Cartridge::from_file("./res/rom/Tetris.gb").unwrap();

    gameboy.load_cartridge(cartridge);

    let mut frontend = Frontend::new();

    while !frontend.should_close() {
        let cycle_info = gameboy.cycle();
        frontend.update_debug_info(cycle_info);
        frontend.set_pixels(gameboy.get_pixels());
        frontend.render();
        sleep(Duration::from_millis(300));
    }
}
