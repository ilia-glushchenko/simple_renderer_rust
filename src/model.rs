use crate::buffer;
use crate::math;
use crate::tech;
use crate::tex;
use std::mem::size_of;
use std::rc::Rc;
use std::vec::Vec;

pub type Vertices = Vec<math::Vec3f>;
pub type Indices = Vec<math::Vec1u>;
pub type Normals = Vec<math::Vec3f>;
pub type Tangents = Vec<math::Vec3f>;
pub type Bitangents = Vec<math::Vec3f>;
pub type UVs = Vec<math::Vec2f>;

#[derive(Clone)]
pub struct MeshAttribute {
    pub name: String,
    pub dimensions: i32,
    pub stride: i32,
    pub data_type: u32,
}

#[derive(Clone)]
pub struct HostMesh {
    pub attributes: Vec<MeshAttribute>,
    pub material_index: usize,
    pub vertices: Vertices,
    pub normals: Normals,
    pub tangents: Tangents,
    pub bitangents: Bitangents,
    pub uvs: UVs,
    pub indices: Indices,
}

#[derive(Clone)]
pub struct MaterialProperty<T> {
    pub name: String,
    pub value: T,
}

#[derive(Clone)]
pub struct HostMaterial {
    pub name: String,
    pub properties_1u: Vec<MaterialProperty<math::Vec1u>>,
    pub properties_1f: Vec<MaterialProperty<math::Vec1f>>,
    pub properties_3f: Vec<MaterialProperty<math::Vec3f>>,
    pub properties_samplers: Vec<MaterialProperty<tex::HostTexture>>,
}

impl HostMaterial {
    pub fn empty() -> HostMaterial {
        HostMaterial {
            name: "".to_string(),
            properties_1u: Vec::new(),
            properties_1f: Vec::new(),
            properties_3f: Vec::new(),
            properties_samplers: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct HostModel {
    pub meshes: Rc<Vec<HostMesh>>,
    pub materials: Rc<Vec<HostMaterial>>,
}

pub struct DeviceMesh {
    pub vao: u32,
    pub index_count: u32,
    pub attributes: Vec<MeshAttribute>,
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

#[derive(Clone)]
pub struct SamplerProgramBinding {
    pub binding: u32,
    pub program: u32,
}

#[derive(Clone)]
pub struct TextureSampler {
    pub name: String,
    pub bindings: Vec<SamplerProgramBinding>,
    pub texture: Rc<tex::DeviceTexture>,
}

impl TextureSampler {
    pub fn new(name: &str, texture: Rc<tex::DeviceTexture>) -> TextureSampler {
        TextureSampler {
            name: name.to_string(),
            bindings: Vec::new(),
            texture: texture,
        }
    }
}

#[derive(Clone)]
pub struct DeviceMaterial {
    pub name: String,
    pub properties_1u: Vec<MaterialProperty<tech::Uniform<math::Vec1u>>>,
    pub properties_1f: Vec<MaterialProperty<tech::Uniform<math::Vec1f>>>,
    pub properties_3f: Vec<MaterialProperty<tech::Uniform<math::Vec3f>>>,
    pub properties_samplers: Vec<MaterialProperty<TextureSampler>>,
}

impl DeviceMaterial {
    pub fn set_svec1f(&mut self, name: &str, value: math::Vec1f) -> Result<(), String> {
        if let Some(property) = self.properties_1f.iter_mut().find(|x| x.value.name == name) {
            if property.value.data_location.data.len() == 1 {
                property.value.data_location.data[0] = value;
                return Ok(());
            }
            return Err(format!("Failed to set array property: '{}'", name).to_string());
        }

        Err(format!("Failed to find property: '{}'", name).to_string())
    }

    pub fn set_svec3f(&mut self, name: &str, value: math::Vec3f) -> Result<(), String> {
        if let Some(property) = self.properties_3f.iter_mut().find(|x| x.value.name == name) {
            if property.value.data_location.data.len() == 1 {
                property.value.data_location.data[0] = value;
                return Ok(());
            }
            return Err(format!("Failed to set array property: '{}'", name).to_string());
        }

        Err(format!("Failed to find property: '{}'", name).to_string())
    }
}

pub struct DeviceModel {
    pub meshes: Vec<DeviceMesh>,
    pub materials: Vec<DeviceMaterial>,
}

impl DeviceMaterial {
    pub fn empty() -> DeviceMaterial {
        DeviceMaterial {
            name: "".to_string(),
            properties_1u: Vec::new(),
            properties_1f: Vec::new(),
            properties_3f: Vec::new(),
            properties_samplers: Vec::new(),
        }
    }
}

pub fn create_host_mesh(
    material_index: usize,
    vertices: Vertices,
    normals: Normals,
    tangents: Tangents,
    bitangents: Bitangents,
    uvs: UVs,
    indices: Indices,
) -> HostMesh {
    HostMesh {
        attributes: create_model_attributes(&vertices, &normals, &tangents, &bitangents, &uvs),
        material_index,
        vertices,
        normals,
        tangents,
        bitangents,
        uvs,
        indices,
    }
}

pub fn create_device_model(host_model: &HostModel) -> DeviceModel {
    let mut materials: Vec<DeviceMaterial> = Vec::new();
    for host_material in host_model.materials.iter() {
        materials.push(create_device_material(host_material));
    }

    let mut meshes: Vec<DeviceMesh> = Vec::new();
    for host_mesh in host_model.meshes.iter() {
        meshes.push(create_device_mesh(host_mesh));
    }

    DeviceModel { meshes, materials }
}

fn create_model_attributes(
    vertices: &Vertices,
    normals: &Normals,
    tangents: &Tangents,
    bitangents: &Bitangents,
    uvs: &UVs,
) -> Vec<MeshAttribute> {
    assert!(!vertices.is_empty(), "Model must always have vertices.");

    let mut attributes: Vec<MeshAttribute> = Vec::new();

    if !vertices.is_empty() {
        attributes.push(create_model_attribute(&vertices, "aPosition"));
    }
    if !normals.is_empty() {
        attributes.push(create_model_attribute(&normals, "aNormal"));
    }
    if !tangents.is_empty() {
        attributes.push(create_model_attribute(&tangents, "aTangent"));
    }
    if !bitangents.is_empty() {
        attributes.push(create_model_attribute(&bitangents, "aBitangent"));
    }
    if !uvs.is_empty() {
        attributes.push(create_model_attribute(&uvs, "aUV"));
    }

    attributes
}

fn create_model_attribute<T>(_: &[T], name: &str) -> MeshAttribute
where
    T: math::VecDimensions<T>,
    T: math::VecGLTypeTrait,
{
    MeshAttribute {
        name: name.to_string(),
        dimensions: T::DIMENSIONS as i32,
        stride: size_of::<T>() as i32,
        data_type: <T as math::VecGLTypeTrait>::GL_TYPE,
    }
}

fn create_device_material(material: &HostMaterial) -> DeviceMaterial {
    let properties_1u = {
        let mut properties = Vec::<MaterialProperty<tech::Uniform<math::Vec1u>>>::new();
        for property in &material.properties_1u {
            properties.push(MaterialProperty {
                name: property.name.clone(),
                value: tech::create_new_uniform(&property.name, property.value),
            })
        }
        properties
    };
    let properties_1f = {
        let mut properties = Vec::<MaterialProperty<tech::Uniform<math::Vec1f>>>::new();
        for property in &material.properties_1f {
            properties.push(MaterialProperty {
                name: property.name.clone(),
                value: tech::create_new_uniform(&property.name, property.value),
            })
        }
        properties
    };
    let properties_3f = {
        let mut properties = Vec::<MaterialProperty<tech::Uniform<math::Vec3f>>>::new();
        for property in &material.properties_3f {
            properties.push(MaterialProperty {
                name: property.name.clone(),
                value: tech::create_new_uniform(&property.name, property.value),
            })
        }
        properties
    };
    let properties_samplers = {
        let mut properties = Vec::<MaterialProperty<TextureSampler>>::new();
        for property in &material.properties_samplers {
            properties.push(MaterialProperty {
                name: property.name.clone(),
                value: create_material_texture_sampler(
                    &property.name,
                    &property.value,
                    &tex::create_color_device_texture_descriptor(&property.value),
                ),
            })
        }
        properties
    };

    DeviceMaterial {
        name: material.name.clone(),
        properties_1u,
        properties_1f,
        properties_3f,
        properties_samplers,
    }
}

fn create_material_texture_sampler(
    name: &str,
    host_texture: &tex::HostTexture,
    desc: &tex::DeviceTextureDescriptor,
) -> TextureSampler {
    TextureSampler {
        name: name.to_string(),
        bindings: Vec::new(),
        texture: tex::create_device_texture(host_texture, desc),
    }
}

fn create_device_mesh(mesh: &HostMesh) -> DeviceMesh {
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
