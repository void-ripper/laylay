#[cfg(target_os = "android")]
use winit::{ event_loop::{ControlFlow, EventLoop}, platform::android::{activity::AndroidApp, EventLoopBuilderExtAndroid}};

#[cfg(target_os = "android")]
mod app;
#[cfg(target_os = "android")]
mod state;
#[cfg(target_os = "android")]
mod model;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    let ev_loop = EventLoop::builder()
        .with_android_app(app.clone())
        .build()
        .unwrap();
    let mut myapp = app::App::new();

    ev_loop.set_control_flow(ControlFlow::Poll);

    let ret = ev_loop.run_app(&mut myapp);

    if let Err(e) = ret {
        tracing::error!("{e}");
    }
}