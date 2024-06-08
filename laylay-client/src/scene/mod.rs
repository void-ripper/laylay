use std::sync::Arc;

use camera::Camera;
use drawable::DrawablePtr;
use node::{Node, NodePtr};
use wgpu::Device;

pub mod camera;
pub mod drawable;
pub mod model;
pub mod node;


pub type ScenePtr = Arc<Scene>;

pub struct Scene {
    drawables: Vec<DrawablePtr>,
    root: NodePtr,
    pub camera: Camera,
}

impl Scene {
    pub async fn new(aspect: f32) -> ScenePtr {
        let camera = Camera::perspective( &[0.0, 1.0, 1.0], &[0.0, 0.0, 0.0], aspect, 45.0, 0.1, 100.0);
        let root = Node::new();
        root.add_child(camera.node.clone()).await;

        Arc::new(Self {
            drawables: Vec::new(),
            root: Node::new(),
            camera,
        })
    }

    pub fn update(&self) {}

    pub fn render(&self) {}
}