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
    util::DeviceExt, Buffer, BufferAddress, BufferUsages, Device, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexStepMode,
};

use crate::{
    app::CTX,
    math::matrix::{Matrix, IDENTITY},
};

use super::{
    material::{Material, DEFAULT},
    model::Vertex,
    node::NodePtr,
};

static ID_POOL: AtomicU32 = AtomicU32::new(1);

pub type DrawablePtr = Arc<Drawable>;

pub struct Instances {
    pub nodes: HashMap<u32, NodePtr>,
    pub instance_matrices: Vec<[f32; 16]>,
    pub instance_materials: Vec<Material>,
    pub instance_buffer: Buffer,
    pub instance_material_buffer: Buffer,
}

pub struct Drawable {
    pub id: u32,
    pub instances: Mutex<Instances>,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub vertex_count: u32,
    pub index_count: u32,
}

impl Drawable {
    pub async fn new<'a>(prim: &Primitive<'a>, doc: &gltf::Gltf) -> DrawablePtr {
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

        let ctx = CTX.get().unwrap();
        let state = ctx.state.lock().await;

        let instance_matrices = Vec::new();
        let instance_materials = Vec::new();
        let instance_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_matrices),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            });

        let instance_material_buffer =
            state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Material Buffer"),
                    contents: bytemuck::cast_slice(&instance_materials),
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                });

        let vertex_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        Arc::new(Self {
            id,
            instances: Mutex::new(Instances {
                nodes: HashMap::new(),
                instance_matrices,
                instance_materials,
                instance_buffer,
                instance_material_buffer,
            }),
            vertex_buffer,
            index_buffer,
            vertex_count: vertices.len() as _,
            index_count: indices.len() as _,
        })
    }

    pub async fn add_node(&self, node: NodePtr) {
        let ctx = CTX.get().unwrap();
        let state = ctx.state.lock().await;
        let mut insts = self.instances.lock().await;
        insts.nodes.insert(node.id, node);
        insts.instance_materials.push(DEFAULT);
        insts.instance_matrices.push(IDENTITY);
        insts.instance_buffer =
            state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&insts.instance_matrices),
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                });

        insts.instance_material_buffer =
            state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Material Buffer"),
                    contents: bytemuck::cast_slice(&insts.instance_materials),
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                });
    }

    pub async fn remove_node(&self, node: NodePtr) {}

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
