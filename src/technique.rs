extern crate gl;
use crate::math;
use crate::shader;
use colored::Colorize;
use std::string::String;
use std::vec::Vec;

pub const DEFAULT_UNIFORM_PROGRAM_LOCATION: UniformProgramLocation = UniformProgramLocation {
    location: 0,
    program: 0,
};

pub struct UniformProgramLocation {
    pub location: i32,
    pub program: u32,
}

pub struct Uniform<T> {
    pub name: String,
    pub location: UniformProgramLocation,
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
}

pub fn bind_shader_program_to_technique(
    technique: &mut Technique,
    program: &shader::ShaderProgram,
) {
    bind_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec1f);
    bind_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec1u);
    bind_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec2f);
    bind_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec3f);
    bind_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.mat4x4f);

    bind_uniforms_to_shader_program(program, &mut technique.per_model_uniforms.vec1f);
    bind_uniforms_to_shader_program(program, &mut technique.per_model_uniforms.vec1u);
    bind_uniforms_to_shader_program(program, &mut technique.per_model_uniforms.vec2f);
    bind_uniforms_to_shader_program(program, &mut technique.per_model_uniforms.vec3f);
    bind_uniforms_to_shader_program(program, &mut technique.per_model_uniforms.mat4x4f);
}

pub fn update_per_frame_uniforms(program: &shader::ShaderProgram, uniforms: &Uniforms) {
    for uniform in &uniforms.vec1f {
        if uniform.location.program == program.handle {
            unsafe {
                gl::Uniform1fv(
                    uniform.location.location,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const f32,
                );
            }
        }
    }

    for uniform in &uniforms.vec1u {
        if uniform.location.program == program.handle {
            unsafe {
                gl::Uniform1uiv(
                    uniform.location.location,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const u32,
                );
            }
        }
    }

    for uniform in &uniforms.vec2f {
        if uniform.location.program == program.handle {
            unsafe {
                gl::Uniform2fv(
                    uniform.location.location,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const f32,
                );
            }
        }
    }

    for uniform in &uniforms.vec3f {
        if uniform.location.program == program.handle {
            unsafe {
                gl::Uniform3fv(
                    uniform.location.location,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const f32,
                );
            }
        }
    }

    for uniform in &uniforms.mat4x4f {
        if uniform.location.program == program.handle {
            unsafe {
                gl::UniformMatrix4fv(
                    uniform.location.location,
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
        if uniform.location.program == program.handle {
            unsafe {
                gl::Uniform1fv(
                    uniform.location.location,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const f32,
                );
            }
        }
    }

    if let Some(uniform) = uniforms.vec1u.get(index) {
        if uniform.location.program == program.handle {
            unsafe {
                gl::Uniform1uiv(
                    uniform.location.location,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const u32,
                );
            }
        }
    }

    if let Some(uniform) = uniforms.vec2f.get(index) {
        if uniform.location.program == program.handle {
            unsafe {
                gl::Uniform2fv(
                    uniform.location.location,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const f32,
                );
            }
        }
    }

    if let Some(uniform) = uniforms.vec3f.get(index) {
        if uniform.location.program == program.handle {
            unsafe {
                gl::Uniform3fv(
                    uniform.location.location,
                    uniform.data.len() as i32,
                    uniform.data.as_ptr() as *const f32,
                );
            }
        }
    }

    if let Some(uniform) = uniforms.mat4x4f.get(index) {
        if uniform.location.program == program.handle {
            unsafe {
                gl::UniformMatrix4fv(
                    uniform.location.location,
                    uniform.data.len() as i32,
                    gl::TRUE, // Transposes the matrix, GL uses different major
                    uniform.data.as_ptr() as *const f32,
                );
            }
        }
    }
}

fn bind_uniforms_to_shader_program<T>(
    program: &shader::ShaderProgram,
    uniforms: &mut [Uniform<T>],
) {
    assert!(
        uniforms
            .iter()
            .find(|&x| x.location.program == program.handle)
            .is_none(),
        "Uniforms can only be bound to a program once."
    );

    for uniform in uniforms.iter_mut() {
        let uniform_name = std::ffi::CString::new(uniform.name.clone()).unwrap();
        let location = unsafe { gl::GetUniformLocation(program.handle, uniform_name.as_ptr()) };
        if location == -1 {
            let message = format!(
                "Failed to find {} uniform location in program {}.",
                uniform.name, program.handle
            );
            println!("{}", message.yellow());
        } else {
            uniform.location.program = program.handle;
            uniform.location.location = location;
        }
    }

    uniforms.sort_unstable_by(|a, b| a.location.program.cmp(&b.location.program));
}
