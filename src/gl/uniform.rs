use crate::gl::{shader, tex};
use crate::math;
use std::rc::Rc;
use std::string::String;

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

impl<T> Uniform<T> {
    pub fn new(name: &str, data: Vec<T>) -> Uniform<T> {
        Uniform::<T> {
            name: name.to_string(),
            data_location: UniformDataLoction {
                locations: Vec::new(),
                data,
            },
        }
    }
}

impl<T> PerModelUnifrom<T>
where
    T: std::clone::Clone,
{
    pub fn new(name: &str, data: Vec<Vec<T>>) -> PerModelUnifrom<T> {
        PerModelUnifrom::<T> {
            name: name.to_string(),
            data_locations: data
                .iter()
                .map(|x| UniformDataLoction {
                    locations: Vec::new(),
                    data: x.clone(),
                })
                .collect(),
        }
    }
}

impl TextureSampler {
    pub fn new(name: &str, texture: Rc<tex::DeviceTexture>) -> TextureSampler {
        TextureSampler {
            name: name.to_string(),
            bindings: Vec::new(),
            texture: texture,
        }
    }

    pub fn bind_shader_program(&mut self, program: &shader::ShaderProgram) {
        if let Some(program_texture) = program.samplers.iter().find(|x| x.name == self.name) {
            self.bindings.push(SamplerProgramBinding {
                binding: program_texture.binding,
                program: program.handle,
            });
        }
    }

    pub fn unbind_shader_program(&mut self, program_handle: u32) {
        self.bindings.retain(|b| b.program == program_handle);
    }
}

///////////////////////////////////////////////////////////
/// Bindings
///////////////////////////////////////////////////////////

pub fn bind_shader_program_to_scalar_uniforms<T>(
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
        }
    }
}

pub fn unbind_shader_program_from_scalar_uniforms<T>(
    program_handle: u32,
    uniforms: &mut [Uniform<T>],
) {
    for uniform in uniforms.iter_mut() {
        uniform
            .data_location
            .locations
            .retain(|l| l.program != program_handle);
    }
}

pub fn bind_shader_program_to_scalar_per_model_uniforms<T>(
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
        }
    }
}

pub fn unbind_shader_program_from_scalar_per_model_uniforms<T>(
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

pub fn bind_shader_program_to_texture_samplers(
    program: &shader::ShaderProgram,
    textures: &mut [TextureSampler],
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
        if let Some(program_sampler) = program.samplers.iter().find(|x| x.name == texture.name) {
            texture.bindings.push(SamplerProgramBinding {
                binding: program_sampler.binding,
                program: program.handle,
            });
        }
    }
}

pub fn unbind_shader_program_from_texture_samplers(
    program_handle: u32,
    textures: &mut [TextureSampler],
) {
    for texture in textures.iter_mut() {
        texture.bindings.retain(|b| b.program != program_handle);
    }
}

///////////////////////////////////////////////////////////
/// Checks
///////////////////////////////////////////////////////////

pub fn check_empty_uniforms(uniforms: &Uniforms) -> Result<(), String> {
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

pub fn check_empty_uniform<T>(uniforms: &[Uniform<T>]) -> Result<(), String> {
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

pub fn check_empty_per_model_uniforms(uniforms: &PerModelUniforms) -> Result<(), String> {
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

pub fn check_empty_per_model_uniform<T>(uniforms: &Vec<PerModelUnifrom<T>>) -> Result<(), String> {
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

pub fn check_per_model_uniforms_consistency(uniforms: &PerModelUniforms) -> Result<(), String> {
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

pub fn check_per_model_uniform_consistency<T>(
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

///////////////////////////////////////////////////////////
/// Updates
///////////////////////////////////////////////////////////

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
