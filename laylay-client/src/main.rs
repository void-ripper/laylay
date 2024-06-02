use winit::event_loop::{ControlFlow, EventLoop};

use laylay_client::App;

fn main() {
    let ev_loop = EventLoop::new().unwrap();
    let app = App::new();

    match app {
        Ok(mut app) => {
            ev_loop.set_control_flow(ControlFlow::Poll);

            let ret = ev_loop.run_app(&mut app);

            if let Err(e) = ret {
                tracing::error!("{e}");
            }
        }
        Err(e) => {
            println!("{e}");
        }
    }
}
