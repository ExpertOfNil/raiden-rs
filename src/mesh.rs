use wgpu::util::DeviceExt;

use super::primitives;
use super::renderer::{Instance, Renderer};

pub const DEFAULT_INSTANCE_CAPACITY: usize = 100;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: glam::Vec3,
    color: glam::Vec3,
    normal: glam::Vec3,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
    pub const fn new(position: glam::Vec3, color: glam::Vec3, normal: glam::Vec3) -> Self {
        Self {
            position,
            color,
            normal,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum MeshType {
    Triangle,
    Cube,
    Tetrahedron,
    Sphere,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub edge_indices: Vec<u16>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub instance_capacity: usize,
    pub edge_instance_buffer: wgpu::Buffer,
    pub edge_instance_capacity: usize,
    pub edge_index_buffer: wgpu::Buffer,
}

impl Mesh {
    pub fn realloc_instance_buffer(&mut self, device: &wgpu::Device, new_capacity: usize) {
        while self.instance_capacity < new_capacity {
            self.instance_capacity *= 2;
        }
        self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mesh Instance Buffer"),
            size: (self.instance_capacity * std::mem::size_of::<Instance>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    }

    pub fn new_cube(device: &wgpu::Device) -> Self {
        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(primitives::CUBE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(primitives::CUBE_INDICES),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        let instance_capacity = DEFAULT_INSTANCE_CAPACITY;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cube Instance Buffer"),
            size: (instance_capacity * std::mem::size_of::<Instance>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Edges
        let edge_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Edge Index Buffer"),
            contents: bytemuck::cast_slice(primitives::CUBE_EDGES),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        let edge_instance_capacity = DEFAULT_INSTANCE_CAPACITY;
        let edge_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cube Edge Instance Buffer"),
            size: (edge_instance_capacity * std::mem::size_of::<Instance>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertices: primitives::CUBE_VERTICES.to_vec(),
            vertex_buffer,
            indices: primitives::CUBE_INDICES.to_vec(),
            index_buffer,
            instance_buffer,
            instance_capacity,
            edge_indices: primitives::CUBE_EDGES.to_vec(),
            edge_index_buffer,
            edge_instance_buffer,
            edge_instance_capacity,
        }
    }
}
