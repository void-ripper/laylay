use std::sync::{atomic::{AtomicU64, Ordering}, Arc};

use gltf::Primitive;
use wgpu::{util::DeviceExt, Buffer, Device};

use super::model::{self, Vertex};

static ID_POOL: AtomicU64 = AtomicU64::new(1);

pub type DrawablePtr = Arc<Drawable>;

pub struct Drawable {
    id: u64,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub vertex_count: u32,
    pub index_count: u32,
}

impl Drawable {
    pub fn new(device: &Device, prim: &Primitive, doc: &gltf::Gltf) -> DrawablePtr {
        let id = ID_POOL.fetch_add(1, Ordering::SeqCst);

        let reader = prim.reader(|_| doc.blob.as_ref().map(|a| a.as_slice()));

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        if let Some(pos) = reader.read_positions().zip(reader.read_normals()) {
            for (pos , norm)in pos.0.zip(pos.1) {
                vertices.push(Vertex {
                    position: pos,
                    normal: norm,
                    color: [0.0, 0.0, 1.0],
                });
            }
        }

        if let Some(ind) = reader.read_indices() {
            for ind in ind.into_u32() {
                indices.push(ind);
            }
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Arc::new(Self {
            id,
            vertex_buffer,
            index_buffer,
            vertex_count: vertices.len() as _,
            index_count: indices.len() as _,
        })
    }

}
