use std::{path::PathBuf, sync::Arc};

use laylay_common::{Info, Message, Version};
use mlua::Lua;
use tokio::{net::TcpStream, runtime::Runtime, sync::mpsc};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    context::{render::RenderContext, xr::XrContext},
    errors::ClientError,
    logger::Logger,
};

pub struct App<'a> {
    lua: Lua,
    prikey: laylay_common::SecretKey,
    runtime: Arc<Runtime>,
    state: Option<RenderContext<'a>>,
    xr: Option<XrContext>,
}

impl<'a> App<'a> {
    pub fn new() -> Result<Self, ClientError> {
        let folder = if cfg!(target_os = "android") {
            "/sdcard/Documents/laylay/"
        } else {
            "data/"
        };
        let folder = PathBuf::from(folder);

        if !folder.exists() {
            std::fs::create_dir_all(&folder)?;
        }

        let prikey = laylay_common::get_private_key(folder)?;
        let public = prikey.public_key().to_sec1_bytes();
        let runtime = Arc::new(Runtime::new()?);
        let app = Self {
            lua: Lua::new(),
            prikey: prikey.clone(),
            runtime: runtime.clone(),
            state: None,
            xr: None,
        };

        let ret: Result<(), ClientError> = app.runtime.block_on(async {
            let addr = if cfg!(target_os = "android") {
                "192.168.1.9"
            } else {
                "127.0.0.1"
            };
            let mut stream = TcpStream::connect((addr, 33033)).await?;
            let greeting = Message::Greeting {
                pubkey: public.into(),
                version: Version::get(),
                info: Info::new(),
            };
            laylay_common::write_greeting(&mut stream, &greeting).await?;

            let ret = laylay_common::read_greeting(&mut stream).await?;
            if let Message::Greeting {
                pubkey,
                version,
                info,
            } = ret
            {
                let shared = laylay_common::shared_secret(pubkey, &prikey);
                let (mut rx, mut tx) = stream.into_split();
                let (txch, mut rxch) = mpsc::channel::<Message>(10);

                tracing::subscriber::set_global_default(Logger::new(runtime, txch))?;

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

            Ok(())
        });

        ret?;

        Ok(app)
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let xr = XrContext::new();

        let xr = match xr {
            Ok(xr) => Some(xr),
            Err(e) => {
                tracing::warn!("{e}");
                None
            }
        };

        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        let state = self.runtime.block_on(async {
            let state = RenderContext::new(window).await;
            state
        });
        self.xr = xr;
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
