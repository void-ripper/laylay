use std::sync::Arc;

use camera::Camera;
use drawable::DrawablePtr;
use light::Light;
use node::{Node, NodePtr};
use tokio::sync::RwLock;

pub mod camera;
pub mod drawable;
pub mod model;
pub mod node;
pub mod light;


pub type ScenePtr = Arc<Scene>;

pub struct Scene {
    pub drawables: RwLock<Vec<DrawablePtr>>,
    root: NodePtr,
    pub camera: RwLock<Camera>,
    pub lights: RwLock<Vec<RwLock<Light>>>,
}

impl Scene {
    pub async fn new(aspect: f32) -> ScenePtr {
        let camera = Camera::perspective( &[0.0, 1.0, -5.0], &[0.0, 0.0, 0.0], aspect, 45.0, 0.1, 100.0).await;
        let root = Node::new();
        root.add_child(camera.node.clone()).await;

        Arc::new(Self {
            drawables: RwLock::new(Vec::new()),
            root: Node::new(),
            camera: RwLock::new(camera),
            lights: RwLock::new(Vec::new()),
        })
    }

    pub async fn add_drawable(&self, d: DrawablePtr) {
        self.drawables.write().await.push(d);
    }

    pub fn update(&self) {}

}
