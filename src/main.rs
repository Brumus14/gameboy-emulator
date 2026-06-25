use crate::{bus::Bus, cartridge::Cartridge, cpu::Cpu, gameboy::Gameboy, registers::Registers};

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
mod timer;

fn main() {
    let mut gameboy = Gameboy::new();
    gameboy.cycle();

    let cartridge = Cartridge::from_file("./res/rom/PokemonYellow.gb").unwrap();
}
