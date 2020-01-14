extern crate colored;
extern crate gl_loader;
extern crate glfw;

use glfw::Context;
use std::collections::HashMap;
use std::f32;
use std::path::Path;

mod buffer;
mod input;
mod material;
mod math;
mod models;
mod pass;
mod shader;
mod technique;
mod window;

struct Camera {
    view: math::Mat4x4f,
    pos: math::Vec3f,
    pitch: f32,
    yaw: f32,
    near: f32,
    far: f32,
    aspect: f32,
    fov: f32,
}

fn update_window_size(window: &mut window::Window) {
    let (width, height) = window.handle.get_framebuffer_size();
    window.width = width as u32;
    window.height = height as u32;
}

fn update_hot_reload(
    device_model: &mut models::DeviceModel,
    techniques: &mut HashMap<String, technique::Technique>,
    program: &mut shader::ShaderProgram,
    input_data: &input::Data,
) {
    if let Some(input::Action::Press) = input_data.keys.get(&input::Key::F5) {
        for device_material in &mut device_model.materials {
            material::unbind_shader_program_from_material(device_material, &program);
        }
        for (_, technique) in techniques.iter_mut() {
            technique::unbind_shader_program_from_technique(technique, &program);
        }

        shader::delete_shader_program(program);
        *program = shader::create_shader_program(
            Path::new("shaders/pass_through.vert"),
            Path::new("shaders/pass_through.frag"),
        );

        for (_, technique) in techniques.iter_mut() {
            technique::bind_shader_program_to_technique(technique, &program);
        }
        for device_material in &mut device_model.materials {
            material::bind_shader_program_to_material(device_material, &program);
        }

        println!("Hot reload");
    }
}

fn update_cursor_mode(window: &mut window::Window, input_data: &mut input::Data) {
    if let Some(input::Action::Press) = input_data.keys.get(&input::Key::F) {
        let mode = window.handle.get_cursor_mode();
        match mode {
            glfw::CursorMode::Normal => {
                window.handle.set_cursor_mode(glfw::CursorMode::Disabled);
                input_data.mouse.pos = window.handle.get_cursor_pos();
                input_data.mouse.prev_pos = input_data.mouse.pos;
            }
            glfw::CursorMode::Disabled => {
                window.handle.set_cursor_mode(glfw::CursorMode::Normal);
                input_data.mouse.pos = (0., 0.);
                input_data.mouse.prev_pos = input_data.mouse.pos;
            }
            _ => panic!(),
        };
    }
}

fn load_device_model() -> models::DeviceModel {
    let host_model = models::load_host_model_from_obj(Path::new("data/models/sponza/sponza.obj"));
    let device_model = models::create_device_model(&host_model);

    device_model
}

fn create_mvp_technique(camera: &Camera, transforms: &Vec<math::Mat4x4f>) -> technique::Technique {
    technique::Technique {
        per_frame_uniforms: technique::Uniforms {
            vec1f: Vec::new(),
            vec1u: Vec::new(),
            vec2f: Vec::new(),
            vec3f: Vec::new(),
            mat4x4f: vec![
                technique::Uniform::<math::Mat4x4f> {
                    name: "uProjMat4".to_string(),
                    locations: Vec::new(),
                    data: vec![math::perspective_projection_mat4x4(
                        camera.fov,
                        camera.aspect,
                        camera.near,
                        camera.far,
                    )],
                },
                technique::Uniform::<math::Mat4x4f> {
                    name: "uViewMat4".to_string(),
                    locations: Vec::new(),
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
            mat4x4f: transforms
                .iter()
                .map(|&x| technique::Uniform::<math::Mat4x4f> {
                    name: "uModelMat4".to_string(),
                    locations: Vec::new(),
                    data: vec![x * math::scale_uniform_mat4x4(5.)],
                })
                .collect(),
        },
        textures_2d: Vec::<models::Sampler2d>::new(),
    }
}

fn update_mvp_technique(tech: &mut technique::Technique, camera: &Camera) {
    let view_mat_index = tech
        .per_frame_uniforms
        .mat4x4f
        .iter()
        .position(|x| x.name == "uViewMat4")
        .expect("MVP technique must have uViewMat4");

    let view_mat = tech.per_frame_uniforms.mat4x4f[view_mat_index]
        .data
        .first_mut()
        .expect("uViewMat4 must have a value");

    *view_mat = camera.view;

    let proj_mat_index = tech
        .per_frame_uniforms
        .mat4x4f
        .iter()
        .position(|x| x.name == "uProjMat4")
        .expect("MVP technique must have uProjMat4");

    let proj_mat = tech.per_frame_uniforms.mat4x4f[proj_mat_index]
        .data
        .first_mut()
        .expect("uProjMat4 must have a value");

    *proj_mat =
        math::perspective_projection_mat4x4(camera.fov, camera.aspect, camera.near, camera.far)
}

fn update_camera(camera: &mut Camera, window: &window::Window, input_data: &input::Data) {
    let speed: f32 = 50.;
    let world_forward = math::Vec4f {
        x: 0.,
        y: 0.,
        z: -1.,
        w: 0.,
    };
    let world_right = math::Vec4f {
        x: -1.,
        y: 0.,
        z: 0.,
        w: 0.,
    };

    let mut forward = math::zero_vec4::<f32>();
    let mut right = math::zero_vec4::<f32>();
    let camera_matrix = math::create_camera_mat4x4(camera.pos, camera.yaw, camera.pitch);

    if let Some(input::Action::Press) | Some(input::Action::Repeat) =
        input_data.keys.get(&input::Key::W)
    {
        forward = camera_matrix * world_forward * speed;
    }
    if let Some(input::Action::Press) | Some(input::Action::Repeat) =
        input_data.keys.get(&input::Key::S)
    {
        forward = camera_matrix * -world_forward * speed;
    }
    if let Some(input::Action::Press) | Some(input::Action::Repeat) =
        input_data.keys.get(&input::Key::A)
    {
        right = camera_matrix * world_right * speed;
    }
    if let Some(input::Action::Press) | Some(input::Action::Repeat) =
        input_data.keys.get(&input::Key::D)
    {
        right = camera_matrix * -world_right * speed;
    }
    if let Some(input::Action::Press) | Some(input::Action::Repeat) =
        input_data.keys.get(&input::Key::Q)
    {
        camera.pos.y -= speed;
    }
    if let Some(input::Action::Press) | Some(input::Action::Repeat) =
        input_data.keys.get(&input::Key::E)
    {
        camera.pos.y += speed;
    }

    let x_dist = input_data.mouse.prev_pos.0 - input_data.mouse.pos.0;
    let y_dist = input_data.mouse.prev_pos.1 - input_data.mouse.pos.1;

    let yaw = (x_dist / window.width as f64) as f32;
    let pitch = (y_dist / window.height as f64) as f32;

    camera.pitch += pitch;
    camera.yaw += yaw;
    camera.pos.x += forward.x + right.x;
    camera.pos.y += forward.y + right.y;
    camera.pos.z += forward.z + right.z;
    camera.aspect = window.width as f32 / window.height as f32;

    camera.view = math::create_view_mat4x4(camera.pos, camera.yaw, camera.pitch);
}

fn main_loop(window: &mut window::Window) {
    let mut input_data: input::Data = input::Data {
        keys: HashMap::new(),
        mouse: input::Mouse {
            pos: (0., 0.),
            prev_pos: (0., 0.),
        },
    };

    let mut device_model = load_device_model();
    let transforms = vec![math::identity_mat4x4(); device_model.meshes.len()];
    let mut camera = Camera {
        view: math::identity_mat4x4(),
        pos: math::zero_vec3(),
        pitch: 0.,
        yaw: 0.,
        near: 10.0,
        far: 20000.0,
        aspect: window.width as f32 / window.height as f32,
        fov: f32::consts::PI / 2. * 0.66,
    };
    let mut techniques: HashMap<String, technique::Technique> = HashMap::new();
    techniques.insert(
        "mvp".to_string(),
        create_mvp_technique(&camera, &transforms),
    );

    let mut program = shader::create_shader_program(
        Path::new("shaders/pass_through.vert"),
        Path::new("shaders/pass_through.frag"),
    );

    technique::bind_shader_program_to_technique(
        techniques.get_mut(&"mvp".to_string()).unwrap(),
        &program,
    );

    for device_material in &mut device_model.materials {
        material::bind_shader_program_to_material(device_material, &program);
    }

    for device_mesh in &device_model.meshes {
        shader::bind_device_mesh_to_shader_program(device_mesh, &program);
    }

    while !window.handle.should_close() {
        input::update_input(window, &mut input_data);
        update_hot_reload(
            &mut device_model,
            &mut techniques,
            &mut program,
            &input_data,
        );
        update_window_size(window);
        update_cursor_mode(window, &mut input_data);
        update_camera(&mut camera, window, &input_data);
        update_mvp_technique(techniques.get_mut(&"mvp".to_string()).unwrap(), &camera);

        pass::execute_render_pass(
            &window,
            &program,
            &techniques.get_mut(&"mvp".to_string()).unwrap(),
            &device_model,
        );
        window.handle.swap_buffers();
    }
}

fn main() {
    let mut window = window::initialize_application();

    main_loop(&mut window);

    window::deinitialize_application();
}
