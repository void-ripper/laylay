use std::{
    path::PathBuf,
    sync::{Arc, OnceLock, Weak},
};

use laylay_common::{Info, Message, SecretKey, Version};
use tokio::{
    net::TcpStream,
    runtime::Runtime,
    sync::{mpsc, Mutex, RwLock},
};
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
    scene::{drawable::Drawable, node::Node, Scene, ScenePtr},
};

// fn log(msg: &str) {
//     use std::{fs::OpenOptions, io::Write};
//     let mut file = OpenOptions::new()
//         .create(true)
//         .append(true)
//         .open("/sdcard/Documents/laylay/log.txt")
//         .unwrap();

//     file.write_all(msg.as_bytes()).unwrap();
// }
pub static CTX: OnceLock<Arc<Context<'static>>> = OnceLock::new();

pub struct Context<'a> {
    me: Weak<Context<'a>>,
    prikey: SecretKey,
    pub runtime: Arc<Runtime>,
    pub state: Mutex<RenderContext<'a>>,
    xr: Option<XrContext>,
    scene: RwLock<Option<ScenePtr>>,
}

pub struct App {
    counter: FrameCounter,
    runtime: Arc<Runtime>,
}

impl App {
    pub fn new() -> Result<Self, ClientError> {
        let runtime = Arc::new(Runtime::new().unwrap());
        let app = Self {
            counter: FrameCounter::new(),
            runtime,
        };

        Ok(app)
    }
}

impl ApplicationHandler for App {
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
        let size = window.inner_size();
        let aspect = size.width as f32 / size.height as f32;
        let state = self.runtime.block_on(async {
            let state = RenderContext::new(window).await;

            state
        });
        #[cfg(target_os = "macos")]
        state.window.request_redraw();

        let folder = if cfg!(target_os = "android") {
            "/sdcard/Documents/laylay/"
        } else {
            "data/"
        };
        let folder = PathBuf::from(folder);

        if !folder.exists() {
            std::fs::create_dir_all(&folder).unwrap();
        }

        let prikey = laylay_common::get_private_key(folder).unwrap();
        let ctx = Arc::new_cyclic(|me| Context {
            me: me.clone(),
            prikey: prikey.clone(),
            runtime: self.runtime.clone(),
            xr,
            state: Mutex::new(state),
            scene: RwLock::new(None),
        });
        CTX.set(ctx.clone());

        self.runtime.block_on(async {
            let scene = Scene::new(aspect).await;

            // let (document, buffers, images) = gltf::import("assets/shrine.glb").unwrap();
            let document = gltf::Gltf::open("assets/boid.glb").unwrap();
            let meshes = document.meshes();

            // document.buffers()
            for mesh in meshes {
                tracing::info!("load mesh: {:?}", mesh.name());
                for prim in mesh.primitives() {
                    let drawable = Drawable::new(&prim, &document).await;
                    tracing::info!(
                        "load mesh primitive v({}) i({})",
                        drawable.vertex_count,
                        drawable.index_count
                    );
                    scene.add_drawable(drawable.clone()).await;
                    let node = Node::new();
                    node.set_drawable(drawable).await;
                    scene.root.add_child(node).await;
                }
            }

            *ctx.scene.write().await = Some(scene);
        });
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        tracing::info!("suspended");
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
                let ctx = CTX.get().unwrap();
                let _delta = self.counter.tick();

                tracing::debug!("redraw-requested");

                self.runtime.block_on(async {
                    if let Some(scene) = &*ctx.scene.read().await {
                        scene.update().await;

                        tracing::debug!("redraw-requested render");
                        if let Err(e) = ctx.state.lock().await.render(scene.clone()).await {
                            tracing::error!("{e}");
                        }
                    }

                    tracing::debug!("redraw-requested request_redraw");
                    ctx.state.lock().await.window.request_redraw();
                });
            }
            WindowEvent::Resized(new_size) => {
                let ctx = CTX.get().unwrap();
                self.runtime.block_on(async {
                    ctx.state.lock().await.resize(new_size);

                    let scene = ctx.scene.read().await;
                    if let Some(scene) = &*scene {
                        scene.camera.write().await.resize(new_size);
                    }
                });
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                let ctx = CTX.get().unwrap();
                self.runtime.block_on(async {
                    let scene = ctx.scene.read().await;

                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyQ) => event_loop.exit(),
                        PhysicalKey::Code(KeyCode::KeyA) => {
                            if let Some(scene) = &*scene {
                                let cam = scene.camera.read().await;
                                let mut m = cam.node.transform.write().await;
                                matrix::translate(&mut *m, &[0.1, 0.0, 0.0]);
                            }
                        }
                        PhysicalKey::Code(KeyCode::KeyD) => {
                            if let Some(scene) = &*scene {
                                let cam = scene.camera.read().await;
                                let mut m = cam.node.transform.write().await;
                                matrix::translate(&mut *m, &[-0.1, 0.0, 0.0]);
                            }
                        }
                        PhysicalKey::Code(KeyCode::KeyW) => {
                            if let Some(scene) = &*scene {
                                let cam = scene.camera.read().await;
                                let mut m = cam.node.transform.write().await;
                                matrix::translate(&mut *m, &[0.0, 0.1, 0.0]);
                            }
                        }
                        PhysicalKey::Code(KeyCode::KeyS) => {
                            if let Some(scene) = &*scene {
                                let cam = scene.camera.read().await;
                                let mut m = cam.node.transform.write().await;
                                matrix::translate(&mut *m, &[0.0, -0.1, 0.0]);
                            }
                        }
                        PhysicalKey::Code(KeyCode::KeyZ) => {
                            if let Some(scene) = &*scene {
                                let cam = scene.camera.read().await;
                                let mut m = cam.node.transform.write().await;
                                matrix::translate(&mut *m, &[0.0, 0.0, 0.1]);
                            }
                        }
                        PhysicalKey::Code(KeyCode::KeyX) => {
                            if let Some(scene) = &*scene {
                                let cam = scene.camera.read().await;
                                let mut m = cam.node.transform.write().await;
                                matrix::translate(&mut *m, &[0.0, 0.0, -0.1]);
                            }
                        }
                        PhysicalKey::Code(KeyCode::KeyP) => {
                            if let Some(scene) = &*scene {
                                let cam = scene.camera.read().await;
                                let m = cam.node.transform.read().await;
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
                });
            }
            _ => {}
        }
    }
}
