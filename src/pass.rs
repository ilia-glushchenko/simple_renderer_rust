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

    technique::update_uniforms_bound_to_program(&technique, &program);

    unsafe {
        gl::DrawElements(gl::TRIANGLES, model.index_count, gl::UNSIGNED_INT, null());
    }
}
