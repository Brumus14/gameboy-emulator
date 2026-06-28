use crate::{bus::Bus, cartridge::Cartridge, cpu::Cpu, gameboy::Gameboy, registers::Registers};

mod audio;
mod bus;
mod cartridge;
mod cpu;
mod gameboy;
mod interrupts;
mod joypad;
mod mbc;
mod opcodes;
mod ppu;
mod registers;
mod serial;
mod timer;

fn main() {
    let mut gameboy = Gameboy::new();

    let cartridge = Cartridge::from_file("./res/rom/Tetris.gb").unwrap();

    gameboy.load_cartridge(cartridge);

    loop {
        gameboy.cycle();
    }
}
