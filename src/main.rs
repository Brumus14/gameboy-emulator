use crate::{
    cartridge::Cartridge, cpu::Cpu, gameboy::Gameboy, memory::Memory, registers::Registers,
};

mod cartridge;
mod cpu;
mod gameboy;
mod mbc;
mod memory;
mod opcodes;
mod registers;

fn main() {
    // let mut gameboy = Gameboy::new();
    // gameboy.cycle();

    // let cartridge = Cartridge::from_file("./res/rom/Tetris.gb").unwrap();
}
