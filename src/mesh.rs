use crate::math;

pub type Vertices = Vec<math::Vec3f>;
pub type Indices = Vec<math::Vec1u>;
pub type Normals = Vec<math::Vec3f>;
pub type Tangents = Vec<math::Vec3f>;
pub type Bitangents = Vec<math::Vec3f>;
pub type UVs = Vec<math::Vec2f>;

#[derive(Clone)]
pub struct Attribute {
    pub name: String,
    pub dimensions: i32,
    pub stride: i32,
    pub data_type: u32,
}

#[derive(Clone)]
pub struct HostMesh {
    pub attributes: Vec<Attribute>,
    pub material_index: usize,
    pub vertices: Vertices,
    pub normals: Normals,
    pub tangents: Tangents,
    pub bitangents: Bitangents,
    pub uvs: UVs,
    pub indices: Indices,
}

pub struct DeviceMesh {
    pub vao: u32,
    pub index_count: u32,
    pub attributes: Vec<Attribute>,
    pub vbos: Vec<u32>,
    pub indices: u32,
    pub material_index: usize,
}

impl Drop for DeviceMesh {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(self.vbos.len() as i32, self.vbos.as_ptr());
            gl::DeleteBuffers(1, &self.indices);
        };
    }
}
