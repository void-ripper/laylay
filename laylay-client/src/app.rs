use std::sync::Arc;

use laylay_common::Message;
use mlua::Lua;
use tokio::{net::TcpStream, runtime::Runtime, sync::mpsc};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{context::xr::XrContext, logger::Logger, state::State};


pub struct App<'a> {
    lua: Lua,
    prikey: laylay_common::SecretKey,
    runtime: Arc<Runtime>,
    state: Option<State<'a>>,
    xr: Option<XrContext>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let prikey = laylay_common::get_private_key("data".into()).unwrap();
        let public = prikey.public_key().to_sec1_bytes();
        let runtime = Arc::new(Runtime::new().unwrap());
        let app = Self {
            lua: Lua::new(),
            prikey: prikey.clone(),
            runtime: runtime.clone(),
            state: None,
            xr: None,
        };

        app.runtime.block_on(async {
            let mut stream = TcpStream::connect(("127.0.0.1", 33033)).await.unwrap();
            let greeting = laylay_common::Message::Greeting {
                pubkey: public.into(),
                version: laylay_common::Version::get(),
            };
            laylay_common::write_greeting(&mut stream, &greeting).await.unwrap();

            let ret = laylay_common::read_greeting(&mut stream).await.unwrap();
            if let Message::Greeting { pubkey, version } = ret {
                let shared = laylay_common::shared_secret(pubkey, &prikey);
                let (mut rx, mut tx) = stream.into_split();
                let (txch, mut rxch) = mpsc::channel::<Message>(10);

                tracing::subscriber::set_global_default(Logger::new(runtime, txch)).unwrap();

                let shared0 = shared.clone();
                tokio::spawn(async move {
                    while let Some(msg) = rxch.recv().await {
                        if let Err(e) = laylay_common::write(&shared0, &mut tx, &msg).await {
                            tracing::error!("{e}");
                        }
                    }
                });

                tokio::spawn(async move {
                    loop {
                        let ret = laylay_common::read(&shared, &mut rx).await;
                        match ret {
                            Ok(msg) => {}
                            Err(e) => {
                                tracing::error!("{e}");
                            }
                        }
                    }
                });
            }
        });

        app
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        let state = self.runtime.block_on(async {
            let state = State::new(window).await;
            state
        });
        self.state = Some(state);
        // self.runtime.
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    if let Err(e) = state.render() {
                        tracing::error!("{e}");
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(state) = &mut self.state {
                    state.resize(new_size);
                }
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                tracing::info!("{:?}", event);
                if event.physical_key == PhysicalKey::Code(KeyCode::KeyQ) {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}
