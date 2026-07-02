use glium::winit::event_loop::EventLoop;

use crate::frontend::app::App;

mod core;
mod frontend;

fn main() {
    // let mut gameboy = Gameboy::new();
    // let cartridge = Cartridge::from_file("./res/rom/Tetris.gb").unwrap();
    //
    // gameboy.load_cartridge(cartridge);
    // gameboy.on();

    let event_loop = EventLoop::builder().build().unwrap();
    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app).unwrap();
}
