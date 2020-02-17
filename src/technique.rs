extern crate gl;
use crate::log;
use crate::math;
use crate::model;
use crate::shader;
use std::collections::HashMap;
use std::ptr::null;
use std::string::String;
use std::vec::Vec;

pub type TechniqueMap = HashMap<Techniques, Technique>;

#[derive(PartialEq, Clone)]
pub struct UniformProgramLocation {
    pub location: u32,
    pub program: u32,
}

#[derive(Clone)]
pub struct UniformDataLoction<T> {
    pub locations: Vec<UniformProgramLocation>,
    pub data: Vec<T>,
}

#[derive(Clone)]
pub struct Uniform<T> {
    pub name: String,
    pub data_location: UniformDataLoction<T>,
}

pub struct Uniforms {
    pub vec1f: Vec<Uniform<math::Vec1f>>,
    pub vec1u: Vec<Uniform<math::Vec1u>>,
    pub vec2f: Vec<Uniform<math::Vec2f>>,
    pub vec3f: Vec<Uniform<math::Vec3f>>,
    pub mat4x4f: Vec<Uniform<math::Mat4x4f>>,
}

pub struct PerModelUnifrom<T> {
    pub name: String,
    pub data_locations: Vec<UniformDataLoction<T>>,
}

pub struct PerModelUniforms {
    pub vec1f: Vec<PerModelUnifrom<math::Vec1f>>,
    pub vec1u: Vec<PerModelUnifrom<math::Vec1u>>,
    pub vec2f: Vec<PerModelUnifrom<math::Vec2f>>,
    pub vec3f: Vec<PerModelUnifrom<math::Vec3f>>,
    pub mat4x4f: Vec<PerModelUnifrom<math::Mat4x4f>>,
}

pub struct Technique {
    pub name: String,
    pub per_frame_uniforms: Uniforms,
    pub per_model_uniforms: PerModelUniforms,
    pub textures: Vec<model::TextureSampler>,
}

#[derive(Hash, PartialEq, Clone)]
pub enum Techniques {
    MVP,
    Skybox,
}

impl Eq for Techniques {}

pub fn create_new_uniform<T>(name: &str, data: T) -> Uniform<T> {
    Uniform::<T> {
        name: name.to_string(),
        data_location: UniformDataLoction {
            locations: Vec::new(),
            data: vec![data],
        },
    }
}

pub fn is_technique_valid(technique: &Technique) -> Result<(), String> {
    if let Err(msg) = check_empty_uniforms(&technique.per_frame_uniforms) {
        return Err(format!(
            "Technique '{}' is invalid.\n{}",
            technique.name, msg
        ));
    }

    if let Err(msg) = check_empty_per_model_uniforms(&technique.per_model_uniforms) {
        return Err(format!(
            "Technique '{}' is invalid.\n{}",
            technique.name, msg
        ));
    }

    if let Err(msg) = check_per_model_uniforms_consistency(&technique.per_model_uniforms) {
        return Err(format!(
            "Technique '{}' is inlvalid!\n{}",
            technique.name, msg
        ));
    }

    Ok(())
}

pub fn update_per_frame_uniforms(program: &shader::ShaderProgram, uniforms: &Uniforms) {
    update_per_frame_uniforms_vec1f(program, &uniforms.vec1f);
    update_per_frame_uniforms_vec1u(program, &uniforms.vec1u);
    update_per_frame_uniforms_vec2f(program, &uniforms.vec2f);
    update_per_frame_uniforms_vec3f(program, &uniforms.vec3f);
    update_per_frame_uniforms_mat4x4f(program, &uniforms.mat4x4f);
}

pub fn update_per_frame_uniforms_vec1f(
    program: &shader::ShaderProgram,
    uniforms: &[Uniform<math::Vec1f>],
) {
    for uniform in uniforms {
        if let Some(location) = uniform
            .data_location
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform1fv(
                    location.location as i32,
                    uniform.data_location.data.len() as i32,
                    uniform.data_location.data.as_ptr() as *const f32,
                );
            }
        }
    }
}

pub fn update_per_frame_uniforms_vec1u(
    program: &shader::ShaderProgram,
    uniforms: &[Uniform<math::Vec1u>],
) {
    for uniform in uniforms {
        if let Some(location) = uniform
            .data_location
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform1uiv(
                    location.location as i32,
                    uniform.data_location.data.len() as i32,
                    uniform.data_location.data.as_ptr() as *const u32,
                );
            }
        }
    }
}

pub fn update_per_frame_uniforms_vec2f(
    program: &shader::ShaderProgram,
    uniforms: &[Uniform<math::Vec2f>],
) {
    for uniform in uniforms {
        if let Some(location) = uniform
            .data_location
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform2fv(
                    location.location as i32,
                    uniform.data_location.data.len() as i32,
                    uniform.data_location.data.as_ptr() as *const f32,
                );
            }
        }
    }
}

pub fn update_per_frame_uniforms_vec3f(
    program: &shader::ShaderProgram,
    uniforms: &[Uniform<math::Vec3f>],
) {
    for uniform in uniforms {
        if let Some(location) = uniform
            .data_location
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform3fv(
                    location.location as i32,
                    uniform.data_location.data.len() as i32,
                    uniform.data_location.data.as_ptr() as *const f32,
                );
            }
        }
    }
}

pub fn update_per_frame_uniforms_mat4x4f(
    program: &shader::ShaderProgram,
    uniforms: &[Uniform<math::Mat4x4f>],
) {
    for uniform in uniforms {
        if let Some(location) = uniform
            .data_location
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::UniformMatrix4fv(
                    location.location as i32,
                    uniform.data_location.data.len() as i32,
                    gl::TRUE, // Transposes the matrix, GL uses different major
                    uniform.data_location.data.as_ptr() as *const f32,
                );
            }
        }
    }
}

pub fn update_per_model_uniform(
    program: &shader::ShaderProgram,
    uniforms: &PerModelUniforms,
    index: usize,
) {
    for uniform in &uniforms.vec1f {
        if let Some(location) = uniform.data_locations[index]
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform1fv(
                    location.location as i32,
                    uniform.data_locations[index].data.len() as i32,
                    uniform.data_locations[index].data.as_ptr() as *const f32,
                );
            }
        }
    }

    for uniform in &uniforms.vec1u {
        if let Some(location) = uniform.data_locations[index]
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform1uiv(
                    location.location as i32,
                    uniform.data_locations[index].data.len() as i32,
                    uniform.data_locations[index].data.as_ptr() as *const u32,
                );
            }
        }
    }

    for uniform in &uniforms.vec2f {
        if let Some(location) = uniform.data_locations[index]
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform2fv(
                    location.location as i32,
                    uniform.data_locations[index].data.len() as i32,
                    uniform.data_locations[index].data.as_ptr() as *const f32,
                );
            }
        }
    }

    for uniform in &uniforms.vec3f {
        if let Some(location) = uniform.data_locations[index]
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform3fv(
                    location.location as i32,
                    uniform.data_locations[index].data.len() as i32,
                    uniform.data_locations[index].data.as_ptr() as *const f32,
                );
            }
        }
    }

    for uniform in &uniforms.mat4x4f {
        if let Some(location) = uniform.data_locations[index]
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::UniformMatrix4fv(
                    location.location as i32,
                    uniform.data_locations[index].data.len() as i32,
                    gl::TRUE, // Transposes the matrix, GL uses different major
                    uniform.data_locations[index].data.as_ptr() as *const f32,
                );
            }
        }
    }
}

pub fn bind_shader_program_to_technique(
    technique: &mut Technique,
    program: &shader::ShaderProgram,
) {
    bind_scalar_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec1f);
    bind_scalar_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec1u);
    bind_scalar_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec2f);
    bind_scalar_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec3f);
    bind_scalar_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.mat4x4f);

    bind_scalar_per_model_uniforms_to_shader_program(
        &program,
        &mut technique.per_model_uniforms.vec1f,
    );
    bind_scalar_per_model_uniforms_to_shader_program(
        &program,
        &mut technique.per_model_uniforms.vec1u,
    );
    bind_scalar_per_model_uniforms_to_shader_program(
        &program,
        &mut technique.per_model_uniforms.vec2f,
    );
    bind_scalar_per_model_uniforms_to_shader_program(
        &program,
        &mut technique.per_model_uniforms.vec3f,
    );
    bind_scalar_per_model_uniforms_to_shader_program(
        &program,
        &mut technique.per_model_uniforms.mat4x4f,
    );

    bind_texture_samplers_to_shader_program(program, &mut technique.textures);
}

pub fn unbind_shader_program_from_technique(technique: &mut Technique, program_handle: u32) {
    unbind_scalar_uniforms_from_shader_program(
        program_handle,
        &mut technique.per_frame_uniforms.vec1f,
    );
    unbind_scalar_uniforms_from_shader_program(
        program_handle,
        &mut technique.per_frame_uniforms.vec1u,
    );
    unbind_scalar_uniforms_from_shader_program(
        program_handle,
        &mut technique.per_frame_uniforms.vec2f,
    );
    unbind_scalar_uniforms_from_shader_program(
        program_handle,
        &mut technique.per_frame_uniforms.vec3f,
    );
    unbind_scalar_uniforms_from_shader_program(
        program_handle,
        &mut technique.per_frame_uniforms.mat4x4f,
    );

    unbind_scalar_per_model_uniforms_from_shader_program(
        program_handle,
        &mut technique.per_model_uniforms.vec1f,
    );
    unbind_scalar_per_model_uniforms_from_shader_program(
        program_handle,
        &mut technique.per_model_uniforms.vec1u,
    );
    unbind_scalar_per_model_uniforms_from_shader_program(
        program_handle,
        &mut technique.per_model_uniforms.vec2f,
    );
    unbind_scalar_per_model_uniforms_from_shader_program(
        program_handle,
        &mut technique.per_model_uniforms.vec3f,
    );
    unbind_scalar_per_model_uniforms_from_shader_program(
        program_handle,
        &mut technique.per_model_uniforms.mat4x4f,
    );

    unbind_texture_samplers_from_shader_program(program_handle, &mut technique.textures);
}

pub fn bind_device_mesh_to_shader_program(
    mesh: &model::DeviceMesh,
    program: &shader::ShaderProgram,
) {
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
    material: &mut model::DeviceMaterial,
    program: &shader::ShaderProgram,
) {
    if let Some(sampler) = &mut material.albedo_texture {
        bind_shader_program_to_texture(program, sampler);
    }
    if let Some(sampler) = &mut material.normal_texture {
        bind_shader_program_to_texture(program, sampler);
    }
    if let Some(sampler) = &mut material.bump_texture {
        bind_shader_program_to_texture(program, sampler);
    }
    if let Some(sampler) = &mut material.metallic_texture {
        bind_shader_program_to_texture(program, sampler);
    }
    if let Some(sampler) = &mut material.roughness_texture {
        bind_shader_program_to_texture(program, sampler);
    }

    bind_scalar_uniforms_to_shader_program(
        program,
        std::slice::from_mut(&mut material.albedo_available),
    );
    bind_scalar_uniforms_to_shader_program(
        program,
        std::slice::from_mut(&mut material.normal_available),
    );
    bind_scalar_uniforms_to_shader_program(
        program,
        std::slice::from_mut(&mut material.bump_available),
    );
    bind_scalar_uniforms_to_shader_program(
        program,
        std::slice::from_mut(&mut material.roughness_available),
    );
    bind_scalar_uniforms_to_shader_program(
        program,
        std::slice::from_mut(&mut material.metallic_available),
    );

    bind_scalar_uniforms_to_shader_program(
        program,
        std::slice::from_mut(&mut material.scalar_albedo),
    );
    bind_scalar_uniforms_to_shader_program(
        program,
        std::slice::from_mut(&mut material.scalar_roughness),
    );
    bind_scalar_uniforms_to_shader_program(
        program,
        std::slice::from_mut(&mut material.scalar_metalness),
    );
}

pub fn unbind_shader_program_from_material(
    material: &mut model::DeviceMaterial,
    program_handle: u32,
) {
    if let Some(sampler) = &mut material.albedo_texture {
        unbind_shader_program_from_texture(program_handle, sampler);
    }
    if let Some(sampler) = &mut material.normal_texture {
        unbind_shader_program_from_texture(program_handle, sampler);
    }
    if let Some(sampler) = &mut material.bump_texture {
        unbind_shader_program_from_texture(program_handle, sampler);
    }
    if let Some(sampler) = &mut material.metallic_texture {
        unbind_shader_program_from_texture(program_handle, sampler);
    }
    if let Some(sampler) = &mut material.roughness_texture {
        unbind_shader_program_from_texture(program_handle, sampler);
    }

    unbind_scalar_uniforms_from_shader_program(
        program_handle,
        std::slice::from_mut(&mut material.albedo_available),
    );
    unbind_scalar_uniforms_from_shader_program(
        program_handle,
        std::slice::from_mut(&mut material.normal_available),
    );
    unbind_scalar_uniforms_from_shader_program(
        program_handle,
        std::slice::from_mut(&mut material.bump_available),
    );
    unbind_scalar_uniforms_from_shader_program(
        program_handle,
        std::slice::from_mut(&mut material.roughness_available),
    );
    unbind_scalar_uniforms_from_shader_program(
        program_handle,
        std::slice::from_mut(&mut material.metallic_available),
    );

    unbind_scalar_uniforms_from_shader_program(
        program_handle,
        std::slice::from_mut(&mut material.scalar_albedo),
    );
    unbind_scalar_uniforms_from_shader_program(
        program_handle,
        std::slice::from_mut(&mut material.scalar_roughness),
    );
    unbind_scalar_uniforms_from_shader_program(
        program_handle,
        std::slice::from_mut(&mut material.scalar_metalness),
    );
}

pub fn bind_shader_program_to_texture(
    program: &shader::ShaderProgram,
    sampler: &mut model::TextureSampler,
) {
    if let Some(program_texture) = program
        .samplers
        .iter()
        .find(|x| x.name == sampler.texture.name)
    {
        sampler.bindings.push(model::SamplerProgramBinding {
            binding: program_texture.binding,
            program: program.handle,
        });
    } else {
        log::log_warning(format!(
            "Failed to find sampler with name: '{}' in shader program id: '{}' name: '{}'",
            sampler.texture.name, program.handle, program.name
        ))
    }
}

pub fn unbind_shader_program_from_texture(
    program_handle: u32,
    sampler: &mut model::TextureSampler,
) {
    sampler.bindings.retain(|b| b.program == program_handle);
}

fn bind_scalar_uniforms_to_shader_program<T>(
    program: &shader::ShaderProgram,
    uniforms: &mut [Uniform<T>],
) {
    assert!(
        uniforms
            .iter()
            .find(|&uniform| uniform
                .data_location
                .locations
                .iter()
                .find(|l| l.program == program.handle)
                .is_some())
            .is_none(),
        "Uniforms can only be bound to a program once."
    );

    for uniform in uniforms.iter_mut() {
        if let Some(program_uniform) = program
            .scalar_uniforms
            .iter()
            .find(|x| x.name == uniform.name)
        {
            uniform
                .data_location
                .locations
                .push(UniformProgramLocation {
                    program: program.handle,
                    location: program_uniform.location,
                });
        } else {
            log::log_warning(format!(
                "Failed to find '{}' uniform location in program id: '{}' name: '{}'",
                uniform.name, program.handle, program.name
            ));
        }
    }
}

fn unbind_scalar_uniforms_from_shader_program<T>(program_handle: u32, uniforms: &mut [Uniform<T>]) {
    for uniform in uniforms.iter_mut() {
        uniform
            .data_location
            .locations
            .retain(|l| l.program != program_handle);
    }
}

fn bind_scalar_per_model_uniforms_to_shader_program<T>(
    program: &shader::ShaderProgram,
    uniforms: &mut Vec<PerModelUnifrom<T>>,
) {
    assert!(
        uniforms
            .iter()
            .find(|&uniform| uniform
                .data_locations
                .iter()
                .find(|data_location| data_location
                    .locations
                    .iter()
                    .find(|l| l.program == program.handle)
                    .is_some())
                .is_some())
            .is_none(),
        "Uniforms can only be bound to a program once."
    );

    for uniform in uniforms.iter_mut() {
        if let Some(program_uniform) = program
            .scalar_uniforms
            .iter()
            .find(|x| x.name == uniform.name)
        {
            for data_location in &mut uniform.data_locations {
                data_location.locations.push(UniformProgramLocation {
                    program: program.handle,
                    location: program_uniform.location,
                });
            }
        } else {
            log::log_warning(format!(
                "Technique has a uniform that is not bound! \
                 Failed to find '{}' uniform location in program id: '{}' name: '{}'",
                uniform.name, program.handle, program.name
            ));
        }
    }
}

fn unbind_scalar_per_model_uniforms_from_shader_program<T>(
    program_handle: u32,
    uniforms: &mut Vec<PerModelUnifrom<T>>,
) {
    for uniform in uniforms {
        for data_location in &mut uniform.data_locations {
            data_location
                .locations
                .retain(|l| l.program != program_handle);
        }
    }
}

fn bind_texture_samplers_to_shader_program(
    program: &shader::ShaderProgram,
    textures: &mut [model::TextureSampler],
) {
    assert!(
        textures
            .iter()
            .find(|&sampler| sampler
                .bindings
                .iter()
                .find(|&binding| binding.program == program.handle)
                .is_some())
            .is_none(),
        "Textures can only be bound to a program once."
    );

    for texture in textures.iter_mut() {
        if let Some(program_sampler) = program
            .samplers
            .iter()
            .find(|x| x.name == texture.texture.name)
        {
            texture.bindings.push(model::SamplerProgramBinding {
                binding: program_sampler.binding,
                program: program.handle,
            });
        } else {
            log::log_warning(format!(
                "Failed to find '{}' sampler location in program id: '{}' name: '{}'",
                texture.texture.name, program.handle, program.name
            ));
        }
    }
}

fn unbind_texture_samplers_from_shader_program(
    program_handle: u32,
    textures: &mut [model::TextureSampler],
) {
    for texture in textures.iter_mut() {
        texture.bindings.retain(|b| b.program != program_handle);
    }
}

fn check_empty_uniforms(uniforms: &Uniforms) -> Result<(), String> {
    if let Err(msg) = check_empty_uniform(&uniforms.vec1f) {
        return Err(msg);
    }
    if let Err(msg) = check_empty_uniform(&uniforms.vec1u) {
        return Err(msg);
    }
    if let Err(msg) = check_empty_uniform(&uniforms.vec2f) {
        return Err(msg);
    }
    if let Err(msg) = check_empty_uniform(&uniforms.vec3f) {
        return Err(msg);
    }
    if let Err(msg) = check_empty_uniform(&uniforms.mat4x4f) {
        return Err(msg);
    }

    Ok(())
}

fn check_empty_uniform<T>(uniforms: &[Uniform<T>]) -> Result<(), String> {
    for (i, uniform) in uniforms.iter().enumerate() {
        if uniform.data_location.data.is_empty() {
            return Err(format!(
                "Uniform '{}' does not have any data.",
                uniform.name
            ));
        }
        if uniform.name.is_empty() {
            return Err(format!("Uniform #'{}' has empty name.", i));
        }
    }

    Ok(())
}

fn check_empty_per_model_uniforms(uniforms: &PerModelUniforms) -> Result<(), String> {
    if let Err(msg) = check_empty_per_model_uniform(&uniforms.vec1f) {
        return Err(msg);
    }
    if let Err(msg) = check_empty_per_model_uniform(&uniforms.vec1u) {
        return Err(msg);
    }
    if let Err(msg) = check_empty_per_model_uniform(&uniforms.vec2f) {
        return Err(msg);
    }
    if let Err(msg) = check_empty_per_model_uniform(&uniforms.vec3f) {
        return Err(msg);
    }
    if let Err(msg) = check_empty_per_model_uniform(&uniforms.mat4x4f) {
        return Err(msg);
    }

    Ok(())
}

fn check_empty_per_model_uniform<T>(uniforms: &Vec<PerModelUnifrom<T>>) -> Result<(), String> {
    for (i, uniform) in uniforms.iter().enumerate() {
        if uniform.name.is_empty() {
            return Err(format!("Uniform #'{}' has empty name.", i));
        }

        for data_location in &uniform.data_locations {
            if data_location.data.is_empty() {
                return Err(format!(
                    "Per model uniform '{}' has empty data at index {}.",
                    uniform.name, i
                ));
            }
        }
    }

    Ok(())
}

fn check_per_model_uniforms_consistency(uniforms: &PerModelUniforms) -> Result<(), String> {
    if let Err(msg) = check_per_model_uniform_consistency(&uniforms.vec1f) {
        return Err(msg);
    }
    if let Err(msg) = check_per_model_uniform_consistency(&uniforms.vec1u) {
        return Err(msg);
    }
    if let Err(msg) = check_per_model_uniform_consistency(&uniforms.vec2f) {
        return Err(msg);
    }
    if let Err(msg) = check_per_model_uniform_consistency(&uniforms.vec3f) {
        return Err(msg);
    }
    if let Err(msg) = check_per_model_uniform_consistency(&uniforms.mat4x4f) {
        return Err(msg);
    }

    Ok(())
}

fn check_per_model_uniform_consistency<T>(
    uniforms: &Vec<PerModelUnifrom<T>>,
) -> Result<(), String> {
    if !uniforms.is_empty() {
        let reference_len = uniforms.first().unwrap().data_locations.len();
        for i in 1..uniforms.len() {
            if uniforms[i].data_locations.len() != reference_len {
                return Err("Per model uniforms have inconsistent lengths.".to_string());
            }
        }
    }

    Ok(())
}
