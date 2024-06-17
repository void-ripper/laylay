
use wgpu::{util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device, ShaderStages};
use winit::dpi::PhysicalSize;

use crate::math::matrix::{self, Matrix};

use super::node::{Node, NodePtr};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawCamera {
    pub view_proj: [f32; 16],
}

pub struct Camera {
    pub node: NodePtr,
    pub projection: Matrix,
    pub transform: Matrix,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn setup(device: &Device) -> (RawCamera, Buffer, BindGroupLayout, BindGroup) {
        let camera = RawCamera {
            view_proj: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        };
        
        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        (camera, camera_buffer, camera_bind_group_layout, camera_bind_group)
    }
      
    pub async fn perspective( eye: &[f32; 3], target: &[f32; 3], aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
        let node = Node::new();
        let mut m = matrix::new();
        matrix::translate(&mut m, eye);
        matrix::look_at(&mut m, target, &[0.0, 1.0, 0.0]);
        *node.transform.write().await = m;
        let inv = matrix::inverse(&m);

        let mut p = matrix::new();
        matrix::perspective(&mut p, fovy, aspect, znear, zfar);

        #[rustfmt::skip]
        let mut opengl_to_wgpu = [ 
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5,
            0.0, 0.0, 0.0, 1.0,
        ];
        matrix::mul_assign(&mut opengl_to_wgpu, &p);
        matrix::mul_assign(&mut opengl_to_wgpu, &inv);
        
        Self {
            node,
            projection: p,
            transform: opengl_to_wgpu,
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.aspect = size.width as f32 / size.height as f32;
        matrix::perspective(&mut self.projection, self.fovy, self.aspect, self.znear, self.zfar);
    }

    pub async fn update(&mut self) {
        #[rustfmt::skip]
        let mut opengl_to_wgpu = [ 
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5,
            0.0, 0.0, 0.0, 1.0,
        ];
        matrix::mul_assign(&mut opengl_to_wgpu, &self.projection);
        let inv = matrix::inverse(&*self.node.transform.read().await);
        matrix::mul_assign(&mut opengl_to_wgpu, &inv);

        self.transform = opengl_to_wgpu;
    }
}
