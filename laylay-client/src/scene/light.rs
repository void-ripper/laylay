use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device};

use crate::math::matrix::{self, Matrix, IDENTITY};

use super::node::{Node, NodePtr};

#[repr(u8)]
pub enum LightKind {
    Off = 0,
    Directional = 1,
    Spot = 2,
    Point = 3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawLight {
    pub kind: u32,
    pub cut_off: f32,
    pub position: [f32; 3],
    pub dir: [f32; 3],
    pub ambient: [f32; 3],
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub attenutation: [f32; 3],
}

pub struct Light {
    pub node: NodePtr,
    pub target: Option<NodePtr>,
    pub projection: Matrix,
    pub size: f32,
    pub raw: RawLight,
}

impl Light {
    pub fn new() -> Self {
        let mut proj = IDENTITY.clone();
        let size = 40.0;
        matrix::ortho(&mut proj, -size, size, -size, size, 0.01, 20.5);
        Self {
            node: Node::new(),
            projection: proj,
            target: None,
            size,
            raw: RawLight {
                kind: LightKind::Point as u32,
                position: [0.0, 0.0, 0.0],
                dir: [0.0, 0.0, 0.0],
                ambient: [1.0, 1.0, 1.0],
                diffuse: [1.0, 1.0, 1.0],
                specular: [1.0, 1.0, 1.0],
                cut_off: 0.35,
                attenutation: [1.0, 0.5, 0.0],
            },
        }
    }

    pub async fn update(&mut self) {
        let wt = self.node.world_transform.read().await.clone();
        self.raw.position[0] = wt[12];
        self.raw.position[1] = wt[13];
        self.raw.position[2] = wt[14];
    }

    pub fn setup(device: &Device) -> ([RawLight; 10], Buffer, BindGroupLayout, BindGroup) {
        let raws = [RawLight {
            kind: LightKind::Point as u32,
            position: [0.0, 0.0, 0.0],
            dir: [0.0, 0.0, 0.0],
            ambient: [1.0, 1.0, 1.0],
            diffuse: [1.0, 1.0, 1.0],
            specular: [1.0, 1.0, 1.0],
            cut_off: 0.35,
            attenutation: [1.0, 0.5, 0.0],
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

        (
            raws,
            light_buffer,
            light_bind_group_layout,
            light_bind_group,
        )
    }
}
