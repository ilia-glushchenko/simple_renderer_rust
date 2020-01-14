extern crate gl;
use crate::material;
use crate::models;
use crate::shader;
use crate::technique;
use crate::window;
use std::ptr::null;

pub fn execute_render_pass(
    window: &window::Window,
    program: &shader::ShaderProgram,
    technique: &technique::Technique,
    model: &models::DeviceModel,
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

        material::bind_material(&program, &model.materials[mesh.material_index as usize]);

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

        material::unbind_material(&program, &model.materials[mesh.material_index as usize]);
    }
}
