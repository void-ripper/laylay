use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod model;
mod logger;
mod state;

fn main() {
    let ev_loop = EventLoop::new().unwrap();
    let mut app = app::App::new();

    ev_loop.set_control_flow(ControlFlow::Poll);

    let ret = ev_loop.run_app(&mut app);

    if let Err(e) = ret {
        tracing::error!("{e}");
    }
}
