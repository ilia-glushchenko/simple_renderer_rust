extern crate gl;
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
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::Viewport(0, 0, window.width, window.height);

        gl::BindVertexArray(model.vao);
        gl::UseProgram(program.handle);
    }

    technique::update_per_frame_uniforms(&program, &technique.per_frame_uniforms);
    technique::update_per_model_uniform(&program, &technique.per_model_uniforms, 0);

    unsafe {
        gl::DrawElements(
            gl::TRIANGLES,
            model.index_count * 3,
            gl::UNSIGNED_INT,
            null(),
        );
    }
}
