extern crate gl;
use crate::log;
use crate::math;
use crate::model;
use crate::pass;
use crate::shader;
use crate::technique;
use std::collections::HashMap;
use std::string::String;
use std::vec::Vec;

pub type TechniqueMap = HashMap<Techniques, Technique>;

#[derive(PartialEq)]
pub struct UniformProgramLocation {
    pub location: u32,
    pub program: u32,
}

pub struct Uniform<T> {
    pub name: String,
    pub locations: Vec<UniformProgramLocation>,
    pub data: Vec<T>,
}

pub struct Uniforms {
    pub vec1f: Vec<Uniform<math::Vec1f>>,
    pub vec1u: Vec<Uniform<math::Vec1u>>,
    pub vec2f: Vec<Uniform<math::Vec2f>>,
    pub vec3f: Vec<Uniform<math::Vec3f>>,
    pub mat4x4f: Vec<Uniform<math::Mat4x4f>>,
}

pub struct Technique {
    pub per_frame_uniforms: Uniforms,
    pub per_model_uniforms: Uniforms,
    pub textures_2d: Vec<model::Sampler2d>,
}

#[derive(Hash, PartialEq)]
pub enum Techniques {
    MVP,
}

impl Eq for Techniques {}

pub fn bind_technique_to_render_pass(techniques: &mut technique::TechniqueMap, pass: &pass::Pass) {
    for (_, technique) in techniques.iter_mut() {
        technique::bind_shader_program_to_technique(technique, &pass.program);
    }
}

pub fn unbind_technique_from_render_pass(
    techniques: &mut technique::TechniqueMap,
    pass: &pass::Pass,
) {
    for (_, technique) in techniques.iter_mut() {
        technique::unbind_shader_program_from_technique(technique, &pass.program);
    }
}

pub fn update_per_frame_uniforms(program: &shader::ShaderProgram, uniforms: &Uniforms) {
    for uniform in &uniforms.vec1f {
        if let Some(location) = uniform
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform1fv(
                    location.location as i32,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const f32,
                );
            }
        }
    }

    for uniform in &uniforms.vec1u {
        if let Some(location) = uniform
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform1uiv(
                    location.location as i32,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const u32,
                );
            }
        }
    }

    for uniform in &uniforms.vec2f {
        if let Some(location) = uniform
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform2fv(
                    location.location as i32,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const f32,
                );
            }
        }
    }

    for uniform in &uniforms.vec3f {
        if let Some(location) = uniform
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform3fv(
                    location.location as i32,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const f32,
                );
            }
        }
    }

    for uniform in &uniforms.mat4x4f {
        if let Some(location) = uniform
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::UniformMatrix4fv(
                    location.location as i32,
                    uniform.data.len() as i32,
                    gl::TRUE, // Transposes the matrix, GL uses different major
                    uniform.data.as_ptr() as *const f32,
                );
            }
        }
    }
}

pub fn update_per_model_uniform(
    program: &shader::ShaderProgram,
    uniforms: &Uniforms,
    index: usize,
) {
    if let Some(uniform) = uniforms.vec1f.get(index) {
        if let Some(location) = uniform
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform1fv(
                    location.location as i32,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const f32,
                );
            }
        }
    }

    if let Some(uniform) = uniforms.vec1u.get(index) {
        if let Some(location) = uniform
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform1uiv(
                    location.location as i32,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const u32,
                );
            }
        }
    }

    if let Some(uniform) = uniforms.vec2f.get(index) {
        if let Some(location) = uniform
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform2fv(
                    location.location as i32,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const f32,
                );
            }
        }
    }

    if let Some(uniform) = uniforms.vec3f.get(index) {
        if let Some(location) = uniform
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::Uniform3fv(
                    location.location as i32,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const f32,
                );
            }
        }
    }

    if let Some(uniform) = uniforms.mat4x4f.get(index) {
        if let Some(location) = uniform
            .locations
            .iter()
            .find(|l| l.program == program.handle)
        {
            unsafe {
                gl::UniformMatrix4fv(
                    location.location as i32,
                    uniform.data.len() as i32,
                    gl::TRUE, // Transposes the matrix, GL uses different major
                    uniform.data.as_ptr() as *const f32,
                );
            }
        }
    }
}

//ToDo: Check if all ShaderProgram dependencies are satisfied
fn bind_shader_program_to_technique(technique: &mut Technique, program: &shader::ShaderProgram) {
    bind_scalar_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec1f);
    bind_scalar_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec1u);
    bind_scalar_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec2f);
    bind_scalar_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec3f);
    bind_scalar_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.mat4x4f);

    bind_scalar_uniforms_to_shader_program(program, &mut technique.per_model_uniforms.vec1f);
    bind_scalar_uniforms_to_shader_program(program, &mut technique.per_model_uniforms.vec1u);
    bind_scalar_uniforms_to_shader_program(program, &mut technique.per_model_uniforms.vec2f);
    bind_scalar_uniforms_to_shader_program(program, &mut technique.per_model_uniforms.vec3f);
    bind_scalar_uniforms_to_shader_program(program, &mut technique.per_model_uniforms.mat4x4f);

    bind_texture2d_to_shader_program(program, &mut technique.textures_2d);
}

fn unbind_shader_program_from_technique(
    technique: &mut Technique,
    program: &shader::ShaderProgram,
) {
    unbind_scalar_uniforms_from_shader_program(program, &mut technique.per_frame_uniforms.vec1f);
    unbind_scalar_uniforms_from_shader_program(program, &mut technique.per_frame_uniforms.vec1u);
    unbind_scalar_uniforms_from_shader_program(program, &mut technique.per_frame_uniforms.vec2f);
    unbind_scalar_uniforms_from_shader_program(program, &mut technique.per_frame_uniforms.vec3f);
    unbind_scalar_uniforms_from_shader_program(program, &mut technique.per_frame_uniforms.mat4x4f);

    unbind_scalar_uniforms_from_shader_program(program, &mut technique.per_model_uniforms.vec1f);
    unbind_scalar_uniforms_from_shader_program(program, &mut technique.per_model_uniforms.vec1u);
    unbind_scalar_uniforms_from_shader_program(program, &mut technique.per_model_uniforms.vec2f);
    unbind_scalar_uniforms_from_shader_program(program, &mut technique.per_model_uniforms.vec3f);
    unbind_scalar_uniforms_from_shader_program(program, &mut technique.per_model_uniforms.mat4x4f);

    unbind_texture2d_from_shader_program(program, &mut technique.textures_2d);
}

fn unbind_scalar_uniforms_from_shader_program<T>(
    program: &shader::ShaderProgram,
    uniforms: &mut [Uniform<T>],
) {
    for uniform in uniforms.iter_mut() {
        uniform.locations.retain(|l| l.program != program.handle);
    }
}

fn bind_scalar_uniforms_to_shader_program<T>(
    program: &shader::ShaderProgram,
    uniforms: &mut [Uniform<T>],
) {
    assert!(
        uniforms
            .iter()
            .find(|&uniform| uniform
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
            uniform.locations.push(UniformProgramLocation {
                program: program.handle,
                location: program_uniform.location,
            });
        } else {
            log::log_warning(format!(
                "Failed to find '{}' uniform location in program '{}'",
                uniform.name, program.handle
            ));
        }
    }
}

fn bind_texture2d_to_shader_program(
    program: &shader::ShaderProgram,
    textures: &mut [model::Sampler2d],
) {
    assert!(
        textures
            .iter()
            .find(|&sampler2d| sampler2d
                .bindings
                .iter()
                .find(|&binding| binding.program == program.handle)
                .is_some())
            .is_none(),
        "Textures can only be bound to a program once."
    );

    for texture in textures.iter_mut() {
        if let Some(program_sampler) = program
            .sampler_2d
            .iter()
            .find(|x| x.name == texture.texture.name)
        {
            texture.bindings.push(model::SamplerProgramBinding {
                binding: program_sampler.binding,
                program: program.handle,
            });
        } else {
            log::log_warning(format!(
                "Failed to find '{}' sampler2d location in program '{}'",
                texture.texture.name, program.handle
            ));
        }
    }
}

fn unbind_texture2d_from_shader_program(
    program: &shader::ShaderProgram,
    textures: &mut [model::Sampler2d],
) {
    for texture in textures.iter_mut() {
        texture.bindings.retain(|b| b.program != program.handle);
    }
}
