use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod state;

fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();

    tracing::info!("-- start --");

    let ev_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&ev_loop).unwrap();

    ev_loop.set_control_flow(ControlFlow::Poll);

    let ret = ev_loop.run(move |event, elwt| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } => {
            if window.id() == window_id {
                match event {
                    WindowEvent::CloseRequested => {
                        tracing::info!("exit");
                        elwt.exit();
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    });

    if let Err(e) = ret {
        tracing::error!("{e}");
    }

    tracing::info!("-- end --");
}
