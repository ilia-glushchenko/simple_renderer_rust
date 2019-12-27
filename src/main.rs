extern crate colored;
extern crate gl_loader;
extern crate glfw;

use glfw::Context;
use std::collections::HashMap;
use std::f32;
use std::path::Path;

mod buffer;
mod input;
mod math;
mod models;
mod pass;
mod shader;
mod technique;
mod window;

fn update_input(window: &mut window::Window, input_data: &mut input::Data) {
    input::update_input(window, input_data);

    if let Some(input::Action::Repeat) = input_data.keys.get(&input::Key::W) {
        println!("W");
    }
    if let Some(input::Action::Repeat) = input_data.keys.get(&input::Key::S) {
        println!("S");
    }
    if let Some(input::Action::Repeat) = input_data.keys.get(&input::Key::A) {
        println!("A");
    }
    if let Some(input::Action::Repeat) = input_data.keys.get(&input::Key::D) {
        println!("D");
    }
}

fn load_models() -> models::Models {
    let device_models = vec![
        buffer::create_device_model(
            models::load_host_model_from_obj(Path::new("data/models/bunny/bunny.obj"))
                .last()
                .unwrap(),
        );
        1
    ];

    let transforms = vec![math::identity_mat4x4(); 1];

    models::create_models_container(device_models, transforms)
}

fn create_mvp_techinique(models: &models::Models) -> technique::Technique {
    technique::Technique {
        per_frame_uniforms: technique::Uniforms {
            vec1f: Vec::new(),
            vec1u: Vec::new(),
            vec2f: Vec::new(),
            vec3f: Vec::new(),
            mat4x4f: vec![
                technique::Uniform::<math::Mat4x4f> {
                    name: "uProjMat4".to_string(),
                    location: technique::DEFAULT_UNIFORM_PROGRAM_LOCATION,
                    data: vec![math::perspective_projection_mat4x4(
                        f32::consts::PI / 2.,
                        1.,
                        0.1,
                        1000.,
                    )],
                },
                technique::Uniform::<math::Mat4x4f> {
                    name: "uViewMat4".to_string(),
                    location: technique::DEFAULT_UNIFORM_PROGRAM_LOCATION,
                    data: vec![math::tranlation_mat4x4(math::Vec3f {
                        x: 0.,
                        y: 0.,
                        z: -1.,
                    })],
                },
            ],
        },
        per_model_uniforms: technique::Uniforms {
            vec1f: Vec::new(),
            vec1u: Vec::new(),
            vec2f: Vec::new(),
            vec3f: Vec::new(),
            mat4x4f: models
                .transforms
                .iter()
                .map(|&x| technique::Uniform::<math::Mat4x4f> {
                    name: "uModelMat4".to_string(),
                    location: technique::DEFAULT_UNIFORM_PROGRAM_LOCATION,
                    data: vec![x * math::scale_uniform_mat4x4(5.)],
                })
                .collect(),
        },
    }
}

fn main_loop(window: &mut window::Window) {
    let mut input_data: input::Data = input::Data {
        keys: HashMap::new(),
    };

    let mut models = load_models();
    let mut mvp_technique = create_mvp_techinique(&models);

    models.transforms[0] = math::identity_mat4x4();

    let program = shader::create_shader_program(
        Path::new("shaders/pass_through.vert"),
        Path::new("shaders/pass_through.frag"),
    );

    technique::bind_shader_program_to_technique(&mut mvp_technique, &program);

    shader::bind_device_model_to_shader_program(models.device_models.first().unwrap(), &program);

    while !window.handle.should_close() {
        update_input(window, &mut input_data);

        pass::execute_render_pass(
            &window,
            &program,
            &mvp_technique,
            models.device_models.first().unwrap(),
        );
        window.handle.swap_buffers();
    }
}

fn main() {
    let mut window = window::initialize_application();

    main_loop(&mut window);

    window::deinitialize_application();
}
