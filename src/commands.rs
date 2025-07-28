use super::mesh::MeshType;
use super::renderer::Instance;

#[derive(Debug, Clone)]
pub struct DrawCommand {
    pub mesh_type: MeshType,
    pub instance: Instance,
}

impl DrawCommand {
    pub fn from_instance(mesh_type: MeshType, instance: Instance) -> Self {
        Self {
            mesh_type,
            instance,
        }
    }
}

pub struct DrawCommandBuilder {
    pub mesh_type: MeshType,
    pub position: glam::Vec3,
    pub rotation: glam::Mat3,
    pub scale: f32,
    pub color: glam::Vec4,
}

impl DrawCommandBuilder {
    pub fn new(mesh_type: MeshType) -> Self {
        Self {
            mesh_type,
            position: glam::Vec3::default(),
            rotation: glam::Mat3::default(),
            scale: 1.0,
            color: [1.0, 1.0, 1.0, 1.0].into(),
        }
    }

    pub fn with_position(self, position: glam::Vec3) -> Self {
        Self { position, ..self }
    }

    pub fn with_rotation(self, rotation: glam::Mat3) -> Self {
        Self { rotation, ..self }
    }

    pub fn with_scale(self, scale: f32) -> Self {
        Self { scale, ..self }
    }

    pub fn with_color(self, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            color: glam::Vec4::new(r, g, b, a),
            ..self
        }
    }

    pub fn with_color_u8(self, r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            color: glam::Vec4::new(r as f32, g as f32, b as f32, a as f32) / 255.0,
            ..self
        }
    }

    pub fn build(self) -> DrawCommand {
        let DrawCommandBuilder {
            mesh_type,
            position,
            rotation,
            scale,
            color,
        } = self;

        let rotation = glam::Quat::from_mat3(&rotation);
        let model_matrix = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::splat(scale),
            rotation,
            position,
        );

        DrawCommand {
            mesh_type,
            instance: Instance {
                model_matrix,
                color,
            },
        }
    }
}
