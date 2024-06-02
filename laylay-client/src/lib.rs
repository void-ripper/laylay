#[cfg(target_os = "android")]
use winit::{
    event_loop::{ControlFlow, EventLoop},
    platform::android::{activity::AndroidApp, EventLoopBuilderExtAndroid},
};

mod app;
mod context;
mod errors;
mod logger;
mod model;
mod state;

pub use app::App;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    let myapp = app::App::new();

    match myapp {
        Ok(mut myapp) => {
            let ev_loop = EventLoop::builder()
                .with_android_app(app.clone())
                .build()
                .unwrap();

            ev_loop.set_control_flow(ControlFlow::Poll);

            let ret = ev_loop.run_app(&mut myapp);

            if let Err(e) = ret {
                std::fs::write("/sdcard/Documents/laylay/error.txt", format!("{e}")).unwrap();
            }
        }
        Err(e) => {
            std::fs::write("/sdcard/Documents/laylay/error.txt", format!("{e}")).unwrap();
        }
    }
}
