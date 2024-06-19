use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

pub static DEFAULT: Material = Material {
    ambient: [0.1, 0.1, 0.1],
    diffuse: [0.89, 0.89, 0.89],
    specular: [1.0, 1.0, 1.0],
    shininess: 10.0,
    opacity: 1.0,
};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material {
    ambient: [f32; 3],
    diffuse: [f32; 3],
    specular: [f32; 3],
    shininess: f32,
    opacity: f32,
}

impl Material {
    pub fn instace_desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Material>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 6,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 7,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as BufferAddress,
                    shader_location: 8,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 9]>() as BufferAddress,
                    shader_location: 9,
                    format: VertexFormat::Float32,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 10]>() as BufferAddress,
                    shader_location: 10,
                    format: VertexFormat::Float32,
                },
            ],
        }
    }
}
