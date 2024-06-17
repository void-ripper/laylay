use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device};

use super::node::{Node, NodePtr};

pub enum LightKind {
    Directional = 0,
    Spot = 1,
    Point = 2,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawLight {
    pub kind: u32,
    pub pos: [f32; 3],
    pub color: [f32; 3],
}

pub struct Light {
    pub node: NodePtr,
    pub enabled: bool,
    pub kind: LightKind,
    pub position: [f32; 3],
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub cut_off: f32,
    pub size: f32,
    pub attenutation: [f32; 3],
}

impl Light {
    pub fn new() -> Self {
        Self {
            node: Node::new(),
            enabled: true,
            kind: LightKind::Spot,
            position: [0.0, 0.0, 0.0],
            ambient: [1.0, 1.0, 1.0],
            diffuse: [1.0, 1.0, 1.0],
            specular: [1.0, 1.0, 1.0],
            cut_off: 0.35,
            size: 40.0,
            attenutation: [1.0, 0.5, 0.0],
        }
    }

    pub fn setup(device: &Device) -> ([RawLight; 10], Buffer, BindGroupLayout, BindGroup) {
        let raws = [RawLight {
            kind: 0,
            pos: [0.0, 0.0, 0.0],
            color: [1.0, 1.0, 1.0],
        }; 10];

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light VB"),
            contents: bytemuck::cast_slice(&raws),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        (raws, light_buffer, light_bind_group_layout, light_bind_group)
    }
}
