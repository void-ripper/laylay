use std::sync::Arc;

use wgpu::{util::DeviceExt, Buffer, Device, RenderPass};

use super::model;


pub type DrawablePtr = Arc<Drawable>;

pub struct Drawable {
    vertex_buffer: Buffer,
    vertex_count: u32,
    index_count: u32,
}

impl Drawable {
    pub fn new(device: Device) -> DrawablePtr {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(model::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(model::VERTICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Arc::new(Self {
            vertex_buffer,
            vertex_count: 3,
            index_count: 0,
        })
    }

    pub fn draw<'a>(&'a self, pass: &mut RenderPass<'a>) {
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        // pass.set_index_buffer(self.index_buffer.slice(..), 0..1);
        pass.draw(0..self.vertex_count, 0..1);
        // pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
