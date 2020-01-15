extern crate gl;
use crate::app;
use crate::model;
use crate::shader;
use crate::technique;
use std::ptr::null;

pub fn execute_render_pass(
    window: &app::Window,
    program: &shader::ShaderProgram,
    technique: &technique::Technique,
    model: &model::DeviceModel,
) {
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::ClearDepth(1.0);
        gl::DepthFunc(gl::LESS);
        gl::DepthMask(gl::TRUE);

        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);

        gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        gl::Viewport(0, 0, window.width as i32, window.height as i32);
        gl::UseProgram(program.handle);
    }

    for mesh in &model.meshes {
        unsafe {
            gl::BindVertexArray(mesh.vao);
        }

        bind_material(&program, &model.materials[mesh.material_index as usize]);

        technique::update_per_frame_uniforms(&program, &technique.per_frame_uniforms);
        technique::update_per_model_uniform(&program, &technique.per_model_uniforms, 0);

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                (mesh.index_count * 3) as i32,
                gl::UNSIGNED_INT,
                null(),
            );
        }

        unbind_material(&program, &model.materials[mesh.material_index as usize]);
    }
}

fn bind_material(program: &shader::ShaderProgram, material: &model::DeviceMaterial) {
    bind_texture(program.handle, &material.albedo_texture);
    bind_texture(program.handle, &material.normal_texture);
    bind_texture(program.handle, &material.bump_texture);
    bind_texture(program.handle, &material.metallic_texture);
    bind_texture(program.handle, &material.roughness_texture);
}

fn unbind_material(program: &shader::ShaderProgram, material: &model::DeviceMaterial) {
    unbind_texture(program.handle, &material.albedo_texture);
    unbind_texture(program.handle, &material.normal_texture);
    unbind_texture(program.handle, &material.bump_texture);
    unbind_texture(program.handle, &material.metallic_texture);
    unbind_texture(program.handle, &material.roughness_texture);
}

fn bind_texture(program: u32, texture: &Option<model::Sampler2d>) {
    if let Some(texture) = &texture {
        if let Some(binding) = texture.bindings.iter().find(|x| x.program == program) {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 as u32 + binding.binding);
                gl::BindTexture(gl::TEXTURE_2D, texture.texture.handle);
            }
        }
    }
}

fn unbind_texture(program: u32, texture: &Option<model::Sampler2d>) {
    if let Some(texture) = &texture {
        if let Some(binding) = texture.bindings.iter().find(|x| x.program == program) {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 as u32 + binding.binding);
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }
        }
    }
}
