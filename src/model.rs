use crate::buffer;
use crate::math;
use crate::tech;
use crate::tex;
use std::mem::size_of;
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
pub struct HostMaterial {
    pub name: String,

    pub albedo_available: bool,
    pub normal_available: bool,
    pub bump_available: bool,
    pub metallic_available: bool,
    pub roughness_available: bool,

    pub albedo_texture: Option<tex::HostTexture>,
    pub normal_texture: Option<tex::HostTexture>,
    pub bump_texture: Option<tex::HostTexture>,
    pub metallic_texture: Option<tex::HostTexture>,
    pub roughness_texture: Option<tex::HostTexture>,

    pub scalar_albedo: math::Vec3f,
    pub scalar_roughness: math::Vec1f,
    pub scalar_metalness: math::Vec1f,
}

#[derive(Clone)]
pub struct HostModel {
    pub meshes: Vec<HostMesh>,
    pub materials: Vec<HostMaterial>,
}

#[derive(Clone)]
pub struct DeviceMesh {
    pub vao: u32,
    pub index_count: u32,
    pub attributes: Vec<MeshAttribute>,
    pub vbos: Vec<u32>,
    pub indices: u32,
    pub material_index: usize,
}

#[derive(Clone)]
pub struct SamplerProgramBinding {
    pub binding: u32,
    pub program: u32,
}

#[derive(Clone)]
pub struct TextureSampler {
    pub bindings: Vec<SamplerProgramBinding>,
    pub texture: tex::DeviceTexture,
}

#[derive(Clone)]
pub struct DeviceMaterial {
    pub albedo_available: tech::Uniform<math::Vec1u>,
    pub normal_available: tech::Uniform<math::Vec1u>,
    pub bump_available: tech::Uniform<math::Vec1u>,
    pub metallic_available: tech::Uniform<math::Vec1u>,
    pub roughness_available: tech::Uniform<math::Vec1u>,

    pub albedo_texture: Option<TextureSampler>,
    pub normal_texture: Option<TextureSampler>,
    pub bump_texture: Option<TextureSampler>,
    pub metallic_texture: Option<TextureSampler>,
    pub roughness_texture: Option<TextureSampler>,

    pub scalar_albedo: tech::Uniform<math::Vec3f>,
    pub scalar_roughness: tech::Uniform<math::Vec1f>,
    pub scalar_metalness: tech::Uniform<math::Vec1f>,
}

#[derive(Clone)]
pub struct DeviceModel {
    pub meshes: Vec<DeviceMesh>,
    pub materials: Vec<DeviceMaterial>,
}

pub fn create_empty_host_material() -> HostMaterial {
    HostMaterial {
        name: "".to_string(),

        albedo_available: false,
        normal_available: false,
        bump_available: false,
        metallic_available: false,
        roughness_available: false,

        albedo_texture: None,
        normal_texture: None,
        bump_texture: None,
        metallic_texture: None,
        roughness_texture: None,

        scalar_albedo: math::zero_vec3(),
        scalar_roughness: math::zero_vec1(),
        scalar_metalness: math::zero_vec1(),
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
    for host_material in &host_model.materials {
        materials.push(create_device_material(host_material));
    }

    let mut meshes: Vec<DeviceMesh> = Vec::new();
    for host_mesh in &host_model.meshes {
        meshes.push(create_device_mesh(host_mesh));
    }

    DeviceModel { meshes, materials }
}

pub fn delete_device_model(device_model: DeviceModel) {
    for device_material in device_model.materials {
        delete_device_material(device_material);
    }

    for device_mesh in device_model.meshes {
        delete_device_mesh(device_mesh);
    }
}

#[allow(clippy::ptr_arg)]
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
        attributes.push(create_model_attribute(&vertices, "aPosition".to_string()));
    }
    if !normals.is_empty() {
        attributes.push(create_model_attribute(&normals, "aNormal".to_string()));
    }
    if !tangents.is_empty() {
        attributes.push(create_model_attribute(&tangents, "aTangent".to_string()));
    }
    if !bitangents.is_empty() {
        attributes.push(create_model_attribute(
            &bitangents,
            "aBitangent".to_string(),
        ));
    }
    if !uvs.is_empty() {
        attributes.push(create_model_attribute(&uvs, "aUV".to_string()));
    }

    attributes
}

fn create_model_attribute<T>(_: &[T], name: String) -> MeshAttribute
where
    T: math::VecDimensions<T>,
    T: math::VecGLTypeTrait,
{
    MeshAttribute {
        name,
        dimensions: T::DIMENSIONS as i32,
        stride: size_of::<T>() as i32,
        data_type: <T as math::VecGLTypeTrait>::GL_TYPE,
    }
}

fn create_device_material(material: &HostMaterial) -> DeviceMaterial {
    DeviceMaterial {
        albedo_available: tech::create_new_uniform(
            "uAlbedoAvailableVec1u",
            math::Vec1u {
                x: material.albedo_available as u32,
            },
        ),
        normal_available: tech::create_new_uniform(
            "uNormalAvailableVec1u",
            math::Vec1u {
                x: material.normal_available as u32,
            },
        ),
        bump_available: tech::create_new_uniform(
            "uBumpAvailableVec1u",
            math::Vec1u {
                x: material.bump_available as u32,
            },
        ),
        metallic_available: tech::create_new_uniform(
            "uMetallicAvailableVec1u",
            math::Vec1u {
                x: material.metallic_available as u32,
            },
        ),
        roughness_available: tech::create_new_uniform(
            "uRoughnessAvailableVec1u",
            math::Vec1u {
                x: material.roughness_available as u32,
            },
        ),

        albedo_texture: create_material_texture(
            "uAlbedoMapSampler2D".to_string(),
            &material.albedo_texture,
            &tex::create_color_device_texture_descriptor(&material.albedo_texture),
        ),
        normal_texture: create_material_texture(
            "uNormalMapSampler2D".to_string(),
            &material.normal_texture,
            &tex::create_color_device_texture_descriptor(&material.normal_texture),
        ),
        bump_texture: create_material_texture(
            "uBumpMapSampler2D".to_string(),
            &material.bump_texture,
            &tex::create_color_device_texture_descriptor(&material.bump_texture),
        ),
        metallic_texture: create_material_texture(
            "uMetallicSampler2D".to_string(),
            &material.metallic_texture,
            &tex::create_color_device_texture_descriptor(&material.metallic_texture),
        ),
        roughness_texture: create_material_texture(
            "uRoughnessSampler2D".to_string(),
            &material.roughness_texture,
            &tex::create_color_device_texture_descriptor(&material.roughness_texture),
        ),

        scalar_albedo: tech::create_new_uniform("uScalarAlbedoVec3f", material.scalar_albedo),
        scalar_roughness: tech::create_new_uniform(
            "uScalarRoughnessVec1f",
            material.scalar_roughness,
        ),
        scalar_metalness: tech::create_new_uniform(
            "uScalarMetalnessVec1f",
            material.scalar_metalness,
        ),
    }
}

fn delete_device_material(device_material: DeviceMaterial) {
    if let Some(sampler) = device_material.albedo_texture {
        tex::delete_device_texture(&sampler.texture);
    }
    if let Some(sampler) = device_material.normal_texture {
        tex::delete_device_texture(&sampler.texture);
    }
    if let Some(sampler) = device_material.bump_texture {
        tex::delete_device_texture(&sampler.texture);
    }
    if let Some(sampler) = device_material.metallic_texture {
        tex::delete_device_texture(&sampler.texture);
    }
    if let Some(sampler) = device_material.roughness_texture {
        tex::delete_device_texture(&sampler.texture);
    }
}

fn create_material_texture(
    name: String,
    host_texture: &Option<tex::HostTexture>,
    desc: &Option<tex::DeviceTextureDescriptor>,
) -> Option<TextureSampler> {
    let mut result: Option<TextureSampler> = None;

    if let (Some(host_texture), Some(desc)) = (host_texture, desc) {
        result = Some(TextureSampler {
            bindings: Vec::new(),
            texture: tex::create_device_texture(name, host_texture, desc),
        });
    }

    result
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

fn delete_device_mesh(mesh: DeviceMesh) {
    unsafe {
        gl::DeleteVertexArrays(1, &mesh.vao);
        gl::DeleteBuffers(mesh.vbos.len() as i32, mesh.vbos.as_ptr());
        gl::DeleteBuffers(1, &mesh.indices);
    };
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
