use glium::{
    Display, Surface,
    backend::glutin::SimpleWindowBuilder,
    glutin::{context::PossiblyCurrentContext, surface::WindowSurface},
    winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, EventLoop},
        window::{Window, WindowId},
    },
};

pub struct App {
    window: Window,
    display: Display<WindowSurface>,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let (window, display) = SimpleWindowBuilder::new().build(event_loop);

        Self { window, display }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let mut frame = self.display.draw();
                frame.clear_color(1.0, 0.0, 0.0, 1.0);
                frame.finish().unwrap();
                println!("sussy");

                self.window.request_redraw();
            }
            _ => (),
        }
    }
}
