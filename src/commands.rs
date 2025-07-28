use super::mesh::MeshType;
use super::renderer::Instance;

#[derive(Debug, Clone)]
pub struct DrawCommand {
    pub primitive_type: MeshType,
    pub instance: Instance,
}
