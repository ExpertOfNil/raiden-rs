use super::mesh::Vertex;
pub const CUBE_VERTICES: &[Vertex] = &[
    Vertex::new(
        glam::Vec3::new(1.0, 1.0, 1.0),
        glam::Vec3::new(1.0, 0.0, 0.0),
        glam::Vec3::new(0.577, 0.577, 0.577),
    ),
    Vertex::new(
        glam::Vec3::new(-1.0, 1.0, 1.0),
        glam::Vec3::new(0.0, 0.0, 1.0),
        glam::Vec3::new(-0.577, 0.577, 0.577),
    ),
    Vertex::new(
        glam::Vec3::new(1.0, -1.0, 1.0),
        glam::Vec3::new(1.0, 0.0, 0.0),
        glam::Vec3::new(0.577, -0.577, 0.577),
    ),
    Vertex::new(
        glam::Vec3::new(-1.0, -1.0, 1.0),
        glam::Vec3::new(0.0, 0.0, 1.0),
        glam::Vec3::new(-0.577, -0.577, 0.577),
    ),
    Vertex::new(
        glam::Vec3::new(1.0, 1.0, -1.0),
        glam::Vec3::new(1.0, 0.0, 0.0),
        glam::Vec3::new(0.577, 0.577, -0.577),
    ),
    Vertex::new(
        glam::Vec3::new(-1.0, 1.0, -1.0),
        glam::Vec3::new(0.0, 0.0, 1.0),
        glam::Vec3::new(-0.577, 0.577, -0.577),
    ),
    Vertex::new(
        glam::Vec3::new(1.0, -1.0, -1.0),
        glam::Vec3::new(1.0, 0.0, 0.0),
        glam::Vec3::new(0.577, -0.577, -0.577),
    ),
    Vertex::new(
        glam::Vec3::new(-1.0, -1.0, -1.0),
        glam::Vec3::new(0.0, 0.0, 1.0),
        glam::Vec3::new(-0.577, -0.577, -0.577),
    ),
];

#[rustfmt::skip]
pub const CUBE_INDICES : &[u16] = &[
    // Front
    0, 1, 3,
    0, 3, 2,
    // Back
    5, 4, 6,
    5, 6, 7,
    // Left
    1, 5, 7,
    1, 7, 3,
    // Right
    4, 0, 2,
    4, 2, 6,
    // Top
    4, 5, 1,
    4, 1, 0,
    // Bottom
    7, 6, 2,
    7, 2, 3,
];

#[rustfmt::skip]
pub const CUBE_EDGES : &[u16] = &[
    0, 1,
    1, 3,
    3, 2,
    2, 0,

    4, 5,
    5, 7,
    7, 6,
    6, 4,

    0, 4,
    1, 5,
    2, 6,
    3, 7,
];
