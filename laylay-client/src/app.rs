use std::{fs::OpenOptions, io::Write, path::PathBuf, sync::Arc};

use laylay_common::{Info, Message, SecretKey, Version};
use tokio::{net::TcpStream, runtime::Runtime, sync::mpsc};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    context::{counter::FrameCounter, render::RenderContext, xr::XrContext},
    errors::ClientError,
    logger::Logger,
    math::matrix,
    scene::{drawable::Drawable, Scene, ScenePtr},
};

fn log(msg: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/sdcard/Documents/laylay/log.txt")
        .unwrap();

    file.write_all(msg.as_bytes()).unwrap();
}

pub struct App<'a> {
    prikey: SecretKey,
    counter: FrameCounter,
    runtime: Arc<Runtime>,
    state: Option<RenderContext<'a>>,
    xr: Option<XrContext>,
    scene: Option<ScenePtr>,
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
            prikey: prikey.clone(),
            counter: FrameCounter::new(),
            runtime: runtime.clone(),
            state: None,
            xr: None,
            scene: None,
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
                info: Info::new()?,
            };
            laylay_common::write_greeting(&mut stream, &greeting).await?;

            let ret = laylay_common::read_greeting(&mut stream).await?;
            if let Message::Greeting {
                pubkey,
                version: _,
                info: _,
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
        tracing::info!("resumed application");

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
        let (scene, state) = self.runtime.block_on(async {
            let size = window.inner_size();
            let aspect = size.width as f32 / size.height as f32;
            let state = RenderContext::new(window).await;
            let scene = Scene::new(aspect).await;

            // let (document, buffers, images) = gltf::import("assets/shrine.glb").unwrap();
            let document = gltf::Gltf::open("assets/boid.glb").unwrap();
            let meshes = document.meshes();

            // document.buffers()
            for mesh in meshes {
                tracing::info!("load mesh: {:?}", mesh.name());
                for prim in mesh.primitives() {
                    let drawable = Drawable::new(&state.device, &prim, &document);
                    tracing::info!(
                        "load mesh primitive v({}) i({})",
                        drawable.vertex_count,
                        drawable.index_count
                    );
                    scene.add_drawable(drawable).await;
                }
            }

            (scene, state)
        });
        #[cfg(target_os = "macos")]
        state.window.request_redraw();

        self.xr = xr;
        self.state = Some(state);
        self.scene = Some(scene);
        // self.runtime.
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        tracing::info!("suspended");
        self.state = None;
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    let _delta = self.counter.tick();

                    if let Some(scene) = &self.scene {
                        self.runtime.block_on(async {
                            scene.update().await;
                            
                            if let Err(e) = state.render(scene.clone()).await {
                                tracing::error!("{e}");
                            }
                        });
                    }
                    state.window.request_redraw();
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(state) = &mut self.state {
                    state.resize(new_size);
                }

                if let Some(scene) = &self.scene {
                    scene.camera.blocking_write().resize(new_size);
                }
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyQ) => event_loop.exit(),
                    PhysicalKey::Code(KeyCode::KeyA) => {
                        if let Some(scene) = &self.scene {
                            let cam = scene.camera.blocking_read();
                            let mut m = cam.node.transform.blocking_write();
                            matrix::translate(&mut *m, &[0.1, 0.0, 0.0]);
                        }
                    }
                    PhysicalKey::Code(KeyCode::KeyD) => {
                        if let Some(scene) = &self.scene {
                            let cam = scene.camera.blocking_read();
                            let mut m = cam.node.transform.blocking_write();
                            matrix::translate(&mut *m, &[-0.1, 0.0, 0.0]);
                        }
                    }
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        if let Some(scene) = &self.scene {
                            let cam = scene.camera.blocking_read();
                            let mut m = cam.node.transform.blocking_write();
                            matrix::translate(&mut *m, &[0.0, 0.1, 0.0]);
                        }
                    }
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        if let Some(scene) = &self.scene {
                            let cam = scene.camera.blocking_read();
                            let mut m = cam.node.transform.blocking_write();
                            matrix::translate(&mut *m, &[0.0, -0.1, 0.0]);
                        }
                    }
                    PhysicalKey::Code(KeyCode::KeyZ) => {
                        if let Some(scene) = &self.scene {
                            let cam = scene.camera.blocking_read();
                            let mut m = cam.node.transform.blocking_write();
                            matrix::translate(&mut *m, &[0.0, 0.0, 0.1]);
                        }
                    }
                    PhysicalKey::Code(KeyCode::KeyX) => {
                        if let Some(scene) = &self.scene {
                            let cam = scene.camera.blocking_read();
                            let mut m = cam.node.transform.blocking_write();
                            matrix::translate(&mut *m, &[0.0, 0.0, -0.1]);
                        }
                    }
                    PhysicalKey::Code(KeyCode::KeyP) => {
                        if let Some(scene) = &self.scene {
                            let cam = scene.camera.blocking_read();
                            let m = cam.node.transform.blocking_read();
                            println!("{:?}", m);
                        }
                    }
                    _ => {
                        tracing::info!("{:?}", event);
                    }
                }
                if event.physical_key == PhysicalKey::Code(KeyCode::KeyQ) {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}
