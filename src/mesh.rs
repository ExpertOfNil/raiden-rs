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

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            color: glam::Vec3::ONE,
            normal: glam::Vec3::ZERO,
        }
    }
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

    pub fn new_tetrahedron(device: &wgpu::Device) -> Mesh {
        const N_VERTICES: usize = 4;
        const N_INDICES: usize = 12;

        #[rustfmt::skip]
	let edge_indices: [u16; N_INDICES] = [
        0, 1,
        1, 2,
        2, 0,
        0, 3,
        1, 3,
        2, 3,
    ];

        let a = (8_f32 / 9_f32).sqrt();
        let b = -1.0 / (2.0 * 6_f32.sqrt());
        let c = -(2_f32 / 9_f32).sqrt();
        let d = (2_f32 / 3_f32).sqrt();
        let e = (3_f32 / 8_f32).sqrt();
        let mut vertices = [Vertex::default(); 4];
        // Base vertex aligned with y-axis
        vertices[0].position = glam::vec3(0.0, a, b);
        // Base vertex
        vertices[1].position = glam::vec3(d, c, b);
        // Base vertex
        vertices[2].position = glam::vec3(-d, c, b);
        // Top vertex aligned with z-axis
        vertices[3].position = glam::vec3(0.0, 0.0, e);

        #[rustfmt::skip]
	let indices: [u16; N_INDICES] = [
        0, 1, 2,
        0, 2, 3,
        2, 1, 3,
        1, 0, 3
    ];

        // Create normals
        for v in 0..N_VERTICES {
            let mut v_norm = glam::Vec3::ZERO;
            for i in (0..N_INDICES).step_by(3) {
                // Make sure the current vertex is in this triangle
                let a = indices[i] as usize;
                let b = indices[i + 1] as usize;
                let c = indices[i + 2] as usize;
                if v != a && v != b && v != c {
                    continue;
                }
                // Find the face normal
                let va = vertices[a].position;
                let vb = vertices[b].position;
                let vc = vertices[c].position;
                let n = (vb - va).cross(vc - va);
                v_norm += n
            }
            vertices[v].normal = v_norm.normalize();
        }

        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tetrahedron Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tetrahedron Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        let instance_capacity = DEFAULT_INSTANCE_CAPACITY;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tetrahedron Instance Buffer"),
            size: (instance_capacity * std::mem::size_of::<Instance>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Edges
        let edge_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Tetrahedron Edge Index Buffer"),
            contents: bytemuck::cast_slice(&edge_indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        let edge_instance_capacity = DEFAULT_INSTANCE_CAPACITY;
        let edge_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tetrahedron Edge Instance Buffer"),
            size: (edge_instance_capacity * std::mem::size_of::<Instance>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Mesh {
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
            edge_indices: edge_indices.to_vec(),
            vertex_buffer,
            index_buffer,
            instance_buffer,
            instance_capacity,
            edge_instance_buffer,
            edge_instance_capacity,
            edge_index_buffer,
        }
    }

    /*
    pub fn new_sphere(renderer: ^Renderer, divisions: u32) -> Mesh {
        using linalg
        longitude := 2 * divisions
        latitude := divisions

        n_vertices := int(2 + (latitude - 1) * longitude)
        // 2 tris per quad
        n_indices := int(6 * longitude * (latitude - 1))
        n_edge_indices := int(
            2 * longitude * ((latitude - 1) + longitude + (latitude - 2)),
        )

        vertices := make([dynamic]Vertex, n_vertices)
        indices := make([dynamic]u16, 0, n_indices)
        edge_indices := make([dynamic]u16, 0, n_edge_indices)

        idx := 0
        vertex := &vertices[idx]

        // Top pole
        vertex.position = Vec3{0, 1, 0}
        vertex.normal = normalize(vertex.position)
        top_index := idx
        idx += 1

        // Rings (excluding poles)
        for i in 1 ..< latitude {
            phi := f32(i) * f32(PI) / f32(latitude) // [0, π]
            y := cos(phi)
            r := sin(phi)

            for j in 0 ..< longitude {
                theta := f32(j) * 2.0 * f32(PI) / f32(longitude) // [0, 2π)
                x := r * cos(theta)
                z := r * sin(theta)

                vertex = &vertices[idx]
                vertex.position = Vec3{x, y, z}
                vertex.normal = normalize(vertex.position)
                idx += 1
            }
        }

        // Bottom pole
        vertex = &vertices[idx]
        vertex.position = Vec3{0, -1, 0}
        vertex.normal = normalize(vertex.position)
        bottom_index := idx

        // === Indices ===

        // Top cap
        for j in 0 ..< longitude {
            next := (j + 1) % longitude
            append(&indices, u16(top_index), u16(1 + next), u16(1 + j))
        }

        // Middle quads
        for i in 0 ..< (latitude - 2) {
            row := 1 + i * longitude
            next_row := row + longitude

            for j in 0 ..< longitude {
                next := (j + 1) % longitude

                a := u16(row + j)
                b := u16(row + next)
                c := u16(next_row + j)
                d := u16(next_row + next)

                append(&indices, a, b, c)
                append(&indices, b, d, c)
            }
        }

        // Bottom cap
        base := 1 + (latitude - 2) * longitude
        for j in 0 ..< longitude {
            next := (j + 1) % longitude
            append(&indices, u16(base + j), u16(base + next), u16(bottom_index))
        }

        // === Edge Indices ===
        for j in 0 ..< longitude {
            // Top pole to first ring
            append(&edge_indices, u16(top_index), u16(1 + j))

            // Connect rings vertically
            for i in 0 ..< (latitude - 2) {
                current_ring := 1 + i * longitude
                next_ring := current_ring + longitude
                append(&edge_indices, u16(current_ring + j), u16(next_ring + j))
            }

            // Last ring to bottom pole
            last_ring := 1 + (latitude - 2) * longitude
            append(&edge_indices, u16(last_ring + j), u16(bottom_index))
        }

        // Latitude rings (horizontal circles)
        for i in 1 ..< latitude {
            ring_start := 1 + (i - 1) * longitude
            for j in 0 ..< longitude {
                next := (j + 1) % longitude
                append(&edge_indices, u16(ring_start + j), u16(ring_start + next))
            }
        }

        mesh := Mesh {
            vertices     = vertices,
            indices      = indices,
            edge_indices = edge_indices,
        }
        mesh_create_buffers(&mesh, renderer, n_vertices, n_indices, n_edge_indices)
        return mesh
    }
        */
}
