use crate::gl::{buffer, shader};
use crate::math;
use std::mem::size_of;
use std::ptr::null;

pub type Indices = Vec<math::Vec1u>;
pub type Vertices = Vec<math::Vec3f>;
pub type Normals = Vec<math::Vec3f>;
pub type Tangents = Vec<math::Vec3f>;
pub type Bitangents = Vec<math::Vec3f>;
pub type UVs = Vec<math::Vec2f>;

pub const VERTEX_ATTRIBUTE_NAME: &str = "aPosition";
pub const NORMAL_ATTRIBUTE_NAME: &str = "aNormal";
pub const TANGENT_ATTRIBUTE_NAME: &str = "aTangent";
pub const BITANGENT_ATTRIBUTE_NAME: &str = "aBitangent";
pub const UV_ATTRIBUTE_NAME: &str = "aUV";

#[derive(Clone)]
pub struct Attribute {
    pub name: String,
    pub dimensions: i32,
    pub stride: i32,
    pub data_type: u32,
}

#[derive(Clone)]
pub struct HostMesh {
    pub name: String,
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
    pub name: String,
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

impl Attribute {
    pub fn new<T>(_: &[T], name: &str) -> Attribute
    where
        T: math::VecDimensions<T>,
        T: math::VecGLTypeTrait,
    {
        Attribute {
            name: name.to_string(),
            dimensions: T::DIMENSIONS as i32,
            stride: size_of::<T>() as i32,
            data_type: <T as math::VecGLTypeTrait>::GL_TYPE,
        }
    }
}

impl HostMesh {
    pub fn new(
        name: String,
        material_index: usize,
        vertices: Vertices,
        normals: Normals,
        tangents: Tangents,
        bitangents: Bitangents,
        uvs: UVs,
        indices: Indices,
    ) -> HostMesh {
        HostMesh {
            name,
            attributes: create_mesh_attributes(&vertices, &normals, &tangents, &bitangents, &uvs),
            material_index,
            vertices,
            normals,
            tangents,
            bitangents,
            uvs,
            indices,
        }
    }
}

impl DeviceMesh {
    pub fn new(mesh: &HostMesh) -> DeviceMesh {
        let mut vao: u32 = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao as *mut u32);
            gl::BindVertexArray(vao);
        };

        assert!(
            (mesh.indices.len() % 3) == 0,
            "Index count should be multiple of 3 to render triangles.",
        );

        DeviceMesh {
            name: mesh.name.clone(),
            vao,
            index_count: mesh.indices.len() as u32 / 3,
            attributes: mesh.attributes.clone(),
            vbos: create_device_mesh_vbos(mesh),
            indices: buffer::create_buffer(&buffer::create_buffer_descriptor(
                &mesh.indices,
                gl::ELEMENT_ARRAY_BUFFER,
            ))
            .expect("Failed to create index buffer."),
            material_index: mesh.material_index,
        }
    }

    pub fn bind_shader_program(&self, program: &shader::ShaderProgram) {
        assert!(
            self.vbos.len() == self.attributes.len(),
            "DeviceModel is invalid! There must be Attribute for each VBO.",
        );

        for program_attribute in &program.attributes {
            for (i, device_mesh_attribute) in self.attributes.iter().enumerate() {
                if device_mesh_attribute.name == program_attribute.name {
                    let vbo = self.vbos[i];
                    unsafe {
                        gl::BindVertexArray(self.vao);
                        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                        gl::EnableVertexAttribArray(program_attribute.location);
                        gl::VertexAttribPointer(
                            program_attribute.location,
                            device_mesh_attribute.dimensions,
                            device_mesh_attribute.data_type,
                            gl::FALSE,
                            device_mesh_attribute.stride,
                            null(),
                        );
                    }
                }
            }

            unsafe {
                gl::BindVertexArray(0);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            }
        }
    }
}

fn create_mesh_attributes(
    vertices: &Vertices,
    normals: &Normals,
    tangents: &Tangents,
    bitangents: &Bitangents,
    uvs: &UVs,
) -> Vec<Attribute> {
    assert!(!vertices.is_empty(), "Model must always have vertices.");

    let mut attributes: Vec<Attribute> = Vec::new();

    if !vertices.is_empty() {
        attributes.push(Attribute::new(&vertices, VERTEX_ATTRIBUTE_NAME));
    }
    if !normals.is_empty() {
        attributes.push(Attribute::new(&normals, NORMAL_ATTRIBUTE_NAME));
    }
    if !tangents.is_empty() {
        attributes.push(Attribute::new(&tangents, TANGENT_ATTRIBUTE_NAME));
    }
    if !bitangents.is_empty() {
        attributes.push(Attribute::new(&bitangents, BITANGENT_ATTRIBUTE_NAME));
    }
    if !uvs.is_empty() {
        attributes.push(Attribute::new(&uvs, UV_ATTRIBUTE_NAME));
    }

    attributes
}

fn create_device_mesh_vbos(mesh: &HostMesh) -> Vec<u32> {
    let mut vbos: Vec<u32> = Vec::new();

    if !mesh.vertices.is_empty() {
        vbos.push(
            buffer::create_buffer(&buffer::create_buffer_descriptor(
                &mesh.vertices,
                gl::ARRAY_BUFFER,
            ))
            .expect("Failed to create vertex buffer."),
        );
    }

    if !mesh.normals.is_empty() {
        vbos.push(
            buffer::create_buffer(&buffer::create_buffer_descriptor(
                &mesh.normals,
                gl::ARRAY_BUFFER,
            ))
            .expect("Failed to create normal buffer."),
        );
    }

    if !mesh.tangents.is_empty() {
        vbos.push(
            buffer::create_buffer(&buffer::create_buffer_descriptor(
                &mesh.tangents,
                gl::ARRAY_BUFFER,
            ))
            .expect("Failed to create tangent buffer."),
        );
    }

    if !mesh.bitangents.is_empty() {
        vbos.push(
            buffer::create_buffer(&buffer::create_buffer_descriptor(
                &mesh.bitangents,
                gl::ARRAY_BUFFER,
            ))
            .expect("Failed to create bitangent buffer."),
        );
    }

    if !mesh.uvs.is_empty() {
        vbos.push(
            buffer::create_buffer(&buffer::create_buffer_descriptor(
                &mesh.uvs,
                gl::ARRAY_BUFFER,
            ))
            .expect("Failed to create uv buffer."),
        );
    }

    vbos
}
