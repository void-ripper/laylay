use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use gltf::Primitive;
use tokio::sync::{Mutex, RwLock};
use wgpu::{
    util::DeviceExt, Buffer, BufferAddress, Device, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexStepMode,
};

use crate::math::matrix::Matrix;

use super::{material::Material, model::Vertex, node::NodePtr};

static ID_POOL: AtomicU32 = AtomicU32::new(1);

pub type DrawablePtr = Arc<Drawable>;

pub struct Drawable {
    pub id: u32,
    pub instances: RwLock<HashMap<u32, NodePtr>>,
    pub instance_matrices: Mutex<Vec<[f32; 16]>>,
    pub instance_materials: Mutex<Vec<Material>>,
    pub instance_buffer: Buffer,
    pub instance_material_buffer: Buffer,
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
            for (pos, norm) in pos.0.zip(pos.1) {
                vertices.push(Vertex {
                    position: pos,
                    normal: norm,
                });
            }
        }

        if let Some(ind) = reader.read_indices() {
            for ind in ind.into_u32() {
                indices.push(ind);
            }
        }

        let instance_matrices = Vec::new();
        let instance_materials = Vec::new();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_matrices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let instance_material_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Material Buffer"),
                contents: bytemuck::cast_slice(&instance_materials),
                usage: wgpu::BufferUsages::VERTEX,
            });

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
            instances: RwLock::new(HashMap::new()),
            instance_matrices: Mutex::new(instance_matrices),
            instance_materials: Mutex::new(instance_materials),
            instance_buffer,
            instance_material_buffer,
            vertex_buffer,
            index_buffer,
            vertex_count: vertices.len() as _,
            index_count: indices.len() as _,
        })
    }

    pub async fn update(&self) {
        let insts = self.instances.read().await;
        let mut matrices = self.instance_matrices.lock().await;
        let mut materials = self.instance_materials.lock().await;
        for (id, n) in insts.values().enumerate() {
            matrices[id] = *n.world_transform.read().await;
            materials[id] = n.material.lock().await.unwrap();
        }
    }

    pub fn instace_desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Matrix>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 3,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as BufferAddress,
                    shader_location: 4,
                    format: VertexFormat::Float32x4,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as BufferAddress,
                    shader_location: 5,
                    format: VertexFormat::Float32x4,
                },
            ],
        }
    }
}
