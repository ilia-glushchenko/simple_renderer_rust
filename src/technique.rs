extern crate gl;
use crate::math;
use crate::shader;
use colored::Colorize;
use std::string::String;
use std::vec::Vec;

pub struct UniformProgramLocation {
    pub location: i32,
    pub program: u32,
}

pub struct OwningUniform<T> {
    pub name: String,
    pub location: UniformProgramLocation,
    pub data: Vec<T>,
}

pub struct NonOwningUniform<T> {
    pub name: String,
    pub location: UniformProgramLocation,
    pub data: *const T,
    pub count: u32,
}

pub struct OwningUniforms {
    pub vec1f: Vec<OwningUniform<math::Vec1f>>,
    pub vec1u: Vec<OwningUniform<math::Vec1u>>,
    pub vec2f: Vec<OwningUniform<math::Vec2f>>,
    pub vec3f: Vec<OwningUniform<math::Vec3f>>,
    pub mat4x4f: Vec<OwningUniform<math::Mat4x4f>>,
}

pub struct NonOwningUniforms {
    pub vec1f: Vec<NonOwningUniform<math::Vec1f>>,
    pub vec1u: Vec<NonOwningUniform<math::Vec1u>>,
    pub vec2f: Vec<NonOwningUniform<math::Vec2f>>,
    pub vec3f: Vec<NonOwningUniform<math::Vec3f>>,
    pub mat4x4f: Vec<NonOwningUniform<math::Mat4x4f>>,
}

pub struct Technique {
    pub per_frame_uniforms: OwningUniforms,
    pub per_model_uniforms: NonOwningUniforms,
}

pub fn bind_shader_program_to_technique(
    technique: &mut Technique,
    program: &shader::ShaderProgram,
) {
    bind_owning_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec1f);
    bind_owning_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec1u);
    bind_owning_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec2f);
    bind_owning_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.vec3f);
    bind_owning_uniforms_to_shader_program(program, &mut technique.per_frame_uniforms.mat4x4f);

    bind_non_owning_uniforms_to_shader_program(program, &mut technique.per_model_uniforms.vec1f);
    bind_non_owning_uniforms_to_shader_program(program, &mut technique.per_model_uniforms.vec1u);
    bind_non_owning_uniforms_to_shader_program(program, &mut technique.per_model_uniforms.vec2f);
    bind_non_owning_uniforms_to_shader_program(program, &mut technique.per_model_uniforms.vec3f);
    bind_non_owning_uniforms_to_shader_program(program, &mut technique.per_model_uniforms.mat4x4f);
}

pub fn update_uniforms_bound_to_program(technique: &Technique, program: &shader::ShaderProgram) {
    update_owning_uniforms_bound_to_program(program, &technique.per_frame_uniforms);
    update_nonowning_uniforms_bound_to_program(program, &technique.per_model_uniforms);
}

fn update_owning_uniforms_bound_to_program(
    program: &shader::ShaderProgram,
    owning_uniforms: &OwningUniforms,
) {
    for uniform in &owning_uniforms.vec1f {
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

    for uniform in &owning_uniforms.vec1u {
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

    for uniform in &owning_uniforms.vec2f {
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

    for uniform in &owning_uniforms.vec3f {
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

    for uniform in &owning_uniforms.mat4x4f {
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

fn update_nonowning_uniforms_bound_to_program(
    program: &shader::ShaderProgram,
    nonowning_uniforms: &NonOwningUniforms,
) {
    for uniform in &nonowning_uniforms.vec1f {
        if uniform.location.program == program.handle {
            unsafe {
                gl::Uniform1fv(
                    uniform.location.location,
                    uniform.count as i32,
                    uniform.data as *const f32,
                );
            }
        }
    }

    for uniform in &nonowning_uniforms.vec1u {
        if uniform.location.program == program.handle {
            unsafe {
                gl::Uniform1uiv(
                    uniform.location.location,
                    uniform.count as i32,
                    uniform.data as *const u32,
                );
            }
        }
    }

    for uniform in &nonowning_uniforms.vec2f {
        if uniform.location.program == program.handle {
            unsafe {
                gl::Uniform2fv(
                    uniform.location.location,
                    uniform.count as i32,
                    uniform.data as *const f32,
                );
            }
        }
    }

    for uniform in &nonowning_uniforms.vec3f {
        if uniform.location.program == program.handle {
            unsafe {
                gl::Uniform3fv(
                    uniform.location.location,
                    uniform.count as i32,
                    uniform.data as *const f32,
                );
            }
        }
    }

    for uniform in &nonowning_uniforms.mat4x4f {
        if uniform.location.program == program.handle {
            unsafe {
                gl::UniformMatrix4fv(
                    uniform.location.location,
                    uniform.count as i32,
                    gl::TRUE, // Transposes the matrix, GL uses different major
                    uniform.data as *const f32,
                );
            }
        }
    }
}

fn bind_owning_uniforms_to_shader_program<T>(
    program: &shader::ShaderProgram,
    owning_uniforms: &mut [OwningUniform<T>],
) {
    assert!(
        owning_uniforms
            .iter()
            .find(|&x| x.location.program == program.handle)
            .is_none(),
        "Uniforms can only be bound to a program once."
    );

    for uniform in owning_uniforms.iter_mut() {
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

    owning_uniforms.sort_unstable_by(|a, b| a.location.program.cmp(&b.location.program));
}

//ToDo: This is a duplicate of bind_owning_uniforms_to_shader_program
//      Might want to replace it with a generic trait or something
fn bind_non_owning_uniforms_to_shader_program<T>(
    program: &shader::ShaderProgram,
    non_owning_uniforms: &mut [NonOwningUniform<T>],
) {
    assert!(
        non_owning_uniforms
            .iter()
            .find(|&x| x.location.program == program.handle)
            .is_none(),
        "Uniforms can only be bound to a program once."
    );

    for uniform in non_owning_uniforms.iter_mut() {
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

    non_owning_uniforms.sort_unstable_by(|a, b| a.location.program.cmp(&b.location.program));
}
