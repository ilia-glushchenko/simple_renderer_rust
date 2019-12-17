extern crate gl;
use crate::models;
use crate::shader;
use crate::window;
use std::ptr::null;

pub fn execute_render_pass(
    window: &window::Window,
    program: &shader::ShaderProgram,
    model: &models::DeviceModel,
) {
    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::Viewport(0, 0, window.width, window.height);

        gl::BindVertexArray(model.vao);
        gl::UseProgram(program.handle);
        gl::DrawElements(gl::TRIANGLES, model.index_count, gl::UNSIGNED_INT, null());
    }
}
