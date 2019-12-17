extern crate colored;
extern crate gl_loader;
extern crate glfw;
use colored::*;
use glfw::{Action, Context, Key};
use std::path::Path;
mod buffer;
mod math;
mod models;
mod pass;
mod shader;
mod window;

fn update_input(window: &mut window::Window) {
    window.glfw.poll_events();
    for (_, event) in glfw::flush_messages(&window.events) {
        let message = format!("{:?}", event).to_string();
        println!("{}", message.blue());
        if let glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) = event {
            window.handle.set_should_close(true)
        }
    }
}

fn load_models() -> models::DeviceModel {
    buffer::create_device_model(
        models::load_host_model_from_obj(Path::new("data/bunny/bunny.obj"))
            .last()
            .unwrap(),
    )
}

fn main_loop(window: &mut window::Window) {
    let model = load_models();

    let program = shader::create_shader_program(
        Path::new("shaders/pass_through.vert"),
        Path::new("shaders/pass_through.frag"),
    );

    shader::bind_device_model_to_shader_program(&model, &program);

    while !window.handle.should_close() {
        update_input(window);
        pass::execute_render_pass(&window, &program, &model);
        window.handle.swap_buffers();
    }
}

fn main() {
    let mut window = window::initialize_application();

    main_loop(&mut window);

    window::deinitialize_application();
}
