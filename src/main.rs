extern crate colored;
extern crate gl_loader;
extern crate glfw;

use glfw::Context;
use std::collections::HashMap;
use std::path::Path;

mod buffer;
mod input;
mod math;
mod models;
mod pass;
mod shader;
mod window;

fn update_input(window: &mut window::Window, input_data: &mut input::Data) {
    input::update_input(window, input_data);

    if let Some(input::Action::Repeat) = input_data.keys.get(&input::Key::W) {
        println!("W");
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
    let mut input_data: input::Data = input::Data {
        keys: HashMap::new(),
    };

    let model = load_models();

    let program = shader::create_shader_program(
        Path::new("shaders/pass_through.vert"),
        Path::new("shaders/pass_through.frag"),
    );

    shader::bind_device_model_to_shader_program(&model, &program);

    while !window.handle.should_close() {
        update_input(window, &mut input_data);
        pass::execute_render_pass(&window, &program, &model);
        window.handle.swap_buffers();
    }
}

fn main() {
    let mut window = window::initialize_application();

    main_loop(&mut window);

    window::deinitialize_application();
}
