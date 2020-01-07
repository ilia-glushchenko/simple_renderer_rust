use crate::models;
use crate::shader;

pub fn bind_shader_program_to_material(
    material: &mut models::DeviceMaterial,
    program: &shader::ShaderProgram,
) {
    bind_shader_program_to_texture(program, &mut material.albedo_texture);
    bind_shader_program_to_texture(program, &mut material.normal_texture);
    bind_shader_program_to_texture(program, &mut material.bump_texture);
    bind_shader_program_to_texture(program, &mut material.metallic_texture);
    bind_shader_program_to_texture(program, &mut material.roughness_texture);
}

pub fn bind_material(program: &shader::ShaderProgram, material: &models::DeviceMaterial) {
    bind_texture(program.handle, &material.albedo_texture);
    bind_texture(program.handle, &material.normal_texture);
    bind_texture(program.handle, &material.bump_texture);
    bind_texture(program.handle, &material.metallic_texture);
    bind_texture(program.handle, &material.roughness_texture);
}

pub fn unbind_material(program: &shader::ShaderProgram, material: &models::DeviceMaterial) {
    unbind_texture(program.handle, &material.albedo_texture);
    unbind_texture(program.handle, &material.normal_texture);
    unbind_texture(program.handle, &material.bump_texture);
    unbind_texture(program.handle, &material.metallic_texture);
    unbind_texture(program.handle, &material.roughness_texture);
}

fn bind_texture(program: u32, texture: &Option<models::Sampler2d>) {
    if let Some(texture) = &texture {
        if let Some(binding) = texture.bindings.iter().find(|x| x.program == program) {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 as u32 + binding.binding);
                gl::BindTexture(gl::TEXTURE_2D, texture.texture.handle);
            }
        }
    }
}

fn unbind_texture(program: u32, texture: &Option<models::Sampler2d>) {
    if let Some(texture) = &texture {
        if let Some(binding) = texture.bindings.iter().find(|x| x.program == program) {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 as u32 + binding.binding);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }
        }
    }
}

fn bind_shader_program_to_texture(
    program: &shader::ShaderProgram,
    sampler2d: &mut Option<models::Sampler2d>,
) {
    if let Some(sampler2d) = sampler2d {
        if let Some(program_texture) = program
            .sampler_2d
            .iter()
            .find(|x| x.name == sampler2d.texture.name)
        {
            sampler2d.bindings.push(models::SamplerProgramBinding {
                binding: program_texture.binding,
                program: program.handle,
            });
        }
    }
}
