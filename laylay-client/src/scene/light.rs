use super::node::NodePtr;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawLight {
    pub pos: [f32; 3],
    pub color: [f32; 3],
}

pub struct Light {
    pub node: NodePtr,
    pub enabled: bool,
    pub pos: [f32; 3],
    pub color: [f32; 3],
}

impl Light {}