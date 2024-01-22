use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

mod state;
use state::State;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();

    tracing::info!("-- start --");

    let ev_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&ev_loop).unwrap();
    let mut state = State::new(window).await;

    ev_loop.set_control_flow(ControlFlow::Poll);

    let ret = ev_loop.run(move |event, elwt| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } => {
            if state.window.id() == window_id {
                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        tracing::info!("redraw");
                        if let Err(e) = state.render() {
                            tracing::error!("{e}");
                        }
                    }
                    WindowEvent::Resized(new_size) => {
                        state.resize(*new_size);
                    }
                    WindowEvent::KeyboardInput {
                        device_id,
                        event,
                        is_synthetic,
                    } => {
                        tracing::info!("{:?}", event);
                        if event.physical_key == PhysicalKey::Code(KeyCode::KeyQ) {
                            elwt.exit();
                        }
                    }
                    _ => {}
                }
            }
        }
        Event::DeviceEvent { device_id, event } => {
            // tracing::info!("device-id {:?} {:?}", device_id, event);
            // match event {
            //     DeviceEvent::Key(key) => {
            //     }
            //     _ => {}
            // }
        }
        Event::NewEvents(StartCause::Poll) => {
            if let Err(e) = state.render() {
                tracing::error!("{e}");
            }
        }
        _ => {}
    });

    if let Err(e) = ret {
        tracing::error!("{e}");
    }

    tracing::info!("-- end --");
}
