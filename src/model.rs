use crate::buffer;
use crate::math;
use crate::shader;
use crate::texture;
use std::mem::size_of;
use std::ptr::null;
use std::vec::Vec;

pub type Vertices = Vec<math::Vec3f>;
pub type Indices = Vec<math::Vec1u>;
pub type Normals = Vec<math::Vec3f>;
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
    pub material_index: u32,
    pub vertices: Vertices,
    pub normals: Normals,
    pub uvs: UVs,
    pub indices: Indices,
}

#[derive(Clone)]
pub struct HostMaterial {
    pub name: String,
    pub albedo_texture: Option<texture::HostTexture>,
    pub normal_texture: Option<texture::HostTexture>,
    pub bump_texture: Option<texture::HostTexture>,
    pub metallic_texture: Option<texture::HostTexture>,
    pub roughness_texture: Option<texture::HostTexture>,
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
    pub material_index: u32,
}

#[derive(Clone)]
pub struct SamplerProgramBinding {
    pub binding: u32,
    pub program: u32,
}

#[derive(Clone)]
pub struct Sampler2d {
    pub bindings: Vec<SamplerProgramBinding>,
    pub texture: texture::DeviceTexture,
}

#[derive(Clone)]
pub struct DeviceMaterial {
    pub albedo_texture: Option<Sampler2d>,
    pub normal_texture: Option<Sampler2d>,
    pub bump_texture: Option<Sampler2d>,
    pub metallic_texture: Option<Sampler2d>,
    pub roughness_texture: Option<Sampler2d>,
}

#[derive(Clone)]
pub struct DeviceModel {
    pub meshes: Vec<DeviceMesh>,
    pub materials: Vec<DeviceMaterial>,
}

pub fn create_host_mesh(
    material_index: u32,
    vertices: Vertices,
    normals: Normals,
    uvs: UVs,
    indices: Indices,
) -> HostMesh {
    HostMesh {
        attributes: create_model_attributes(&vertices, &normals, &uvs),
        material_index,
        vertices,
        normals,
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

pub fn bind_device_mesh_to_shader_program(mesh: &DeviceMesh, program: &shader::ShaderProgram) {
    assert!(
        mesh.vbos.len() == mesh.attributes.len(),
        "DeviceModel is invalid! There must be Attribute for each VBO.",
    );

    for program_attribute in &program.attributes {
        for (i, device_mesh_attribute) in mesh.attributes.iter().enumerate() {
            if device_mesh_attribute.name == program_attribute.name {
                let vbo = mesh.vbos[i];
                unsafe {
                    gl::BindVertexArray(mesh.vao);
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

pub fn bind_shader_program_to_material(
    material: &mut DeviceMaterial,
    program: &shader::ShaderProgram,
) {
    bind_shader_program_to_texture(program, &mut material.albedo_texture);
    bind_shader_program_to_texture(program, &mut material.normal_texture);
    bind_shader_program_to_texture(program, &mut material.bump_texture);
    bind_shader_program_to_texture(program, &mut material.metallic_texture);
    bind_shader_program_to_texture(program, &mut material.roughness_texture);
}

pub fn unbind_shader_program_from_material(
    material: &mut DeviceMaterial,
    program: &shader::ShaderProgram,
) {
    unbind_shader_program_from_texture(program, &mut material.albedo_texture);
    unbind_shader_program_from_texture(program, &mut material.normal_texture);
    unbind_shader_program_from_texture(program, &mut material.bump_texture);
    unbind_shader_program_from_texture(program, &mut material.metallic_texture);
    unbind_shader_program_from_texture(program, &mut material.roughness_texture);
}

fn bind_shader_program_to_texture(
    program: &shader::ShaderProgram,
    sampler2d: &mut Option<Sampler2d>,
) {
    if let Some(sampler2d) = sampler2d {
        if let Some(program_texture) = program
            .sampler_2d
            .iter()
            .find(|x| x.name == sampler2d.texture.name)
        {
            sampler2d.bindings.push(SamplerProgramBinding {
                binding: program_texture.binding,
                program: program.handle,
            });
        }
    }
}

fn unbind_shader_program_from_texture(
    program: &shader::ShaderProgram,
    sampler2d: &mut Option<Sampler2d>,
) {
    if let Some(sampler2d) = sampler2d {
        sampler2d.bindings.retain(|b| b.program == program.handle);
    }
}

#[allow(clippy::ptr_arg)]
fn create_model_attributes(
    vertices: &Vertices,
    normals: &Normals,
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
        albedo_texture: create_material_texture(
            "uAlbedoMapSampler2D".to_string(),
            &material.albedo_texture,
            &texture::create_color_device_texture_descriptor(&material.albedo_texture),
        ),
        normal_texture: create_material_texture(
            "uNormalMapSampler2D".to_string(),
            &material.normal_texture,
            &texture::create_color_device_texture_descriptor(&material.normal_texture),
        ),
        bump_texture: create_material_texture(
            "uBumpMapSampler2D".to_string(),
            &material.bump_texture,
            &texture::create_color_device_texture_descriptor(&material.bump_texture),
        ),
        metallic_texture: create_material_texture(
            "uMetallicSampler2D".to_string(),
            &material.metallic_texture,
            &texture::create_color_device_texture_descriptor(&material.metallic_texture),
        ),
        roughness_texture: create_material_texture(
            "uRoughnessSampler2D".to_string(),
            &material.roughness_texture,
            &texture::create_color_device_texture_descriptor(&material.roughness_texture),
        ),
    }
}

fn create_material_texture(
    name: String,
    host_texture: &Option<texture::HostTexture>,
    desc: &Option<texture::DeviceTextureDescriptor>,
) -> Option<Sampler2d> {
    let mut result: Option<Sampler2d> = None;

    if let (Some(host_texture), Some(desc)) = (host_texture, desc) {
        result = Some(Sampler2d {
            bindings: Vec::new(),
            texture: texture::create_device_texture(name, host_texture, desc),
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
