use glfw::Context;
use std::collections::HashMap;
use std::path::Path;

mod app;
mod buffer;
mod camera;
mod helper;
mod input;
mod loader;
mod log;
mod math;
mod model;
mod pass;
mod shader;
mod technique;
mod texture;

fn load_device_model() -> model::DeviceModel {
    let host_model = loader::load_host_model_from_obj(Path::new("data/models/sponza/sponza.obj"));
    let device_model = model::create_device_model(&host_model);

    device_model
}

fn create_mvp_technique(
    camera: &camera::Camera,
    transforms: &Vec<math::Mat4x4f>,
) -> technique::Technique {
    technique::Technique {
        per_frame_uniforms: technique::Uniforms {
            vec1f: Vec::new(),
            vec1u: Vec::new(),
            vec2f: Vec::new(),
            vec3f: vec![technique::Uniform::<math::Vec3f> {
                name: "uCameraPosVec3".to_string(),
                locations: Vec::new(),
                data: vec![camera.pos],
            }],
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
        textures_2d: Vec::<model::Sampler2d>::new(),
    }
}

fn update_mvp_technique(tech: &mut technique::Technique, camera: &camera::Camera) {
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
        math::perspective_projection_mat4x4(camera.fov, camera.aspect, camera.near, camera.far);

    let camera_pos_index = tech
        .per_frame_uniforms
        .vec3f
        .iter()
        .position(|x| x.name == "uCameraPosVec3")
        .expect("MVP technique must have uCameraPosVec3");
    let camera_pos_vec = tech.per_frame_uniforms.vec3f[camera_pos_index]
        .data
        .first_mut()
        .expect("uCameraPosVec3 must have a value");
    *camera_pos_vec = camera.pos;
}

fn create_linghting_pass(window: &app::Window) -> Result<pass::Pass, String> {
    let lighting_pass_descriptor = pass::PassDescriptor {
        name: "lighting".to_string(),
        program: shader::HostShaderProgramDescriptor {
            name: "lighting".to_string(),
            vert_shader_file_path: "shaders/pass_through.vert".to_string(),
            frag_shader_file_path: "shaders/pass_through.frag".to_string(),
        },
        techniques: vec![technique::Techniques::MVP],
        attachments: vec![
            pass::PassAttachmentDescriptor {
                flavor: pass::PassAttachments::Depth(1., gl::LESS),
                clear: true,
                write: true,
                width: window.width,
                height: window.height,
            },
            pass::PassAttachmentDescriptor {
                flavor: pass::PassAttachments::Color(math::Vec4f {
                    x: 1.,
                    y: 1.,
                    z: 1.,
                    w: 1.,
                }),
                clear: true,
                write: true,
                width: window.width,
                height: window.height,
            },
        ],
        width: window.width,
        height: window.height,
    };

    pass::create_render_pass(lighting_pass_descriptor)
}

fn main_loop(window: &mut app::Window) {
    let mut input_data: input::Data = input::Data {
        keys: HashMap::new(),
        mouse: input::Mouse {
            pos: (0., 0.),
            prev_pos: (0., 0.),
        },
    };

    let mut device_model = load_device_model();
    let transforms = vec![math::identity_mat4x4(); device_model.meshes.len()];
    let mut camera = camera::create_default_camera(window.width, window.height);
    let mut techniques = technique::TechniqueMap::new();
    techniques.insert(
        technique::Techniques::MVP,
        create_mvp_technique(&camera, &transforms),
    );

    let mut lighting_pass = create_linghting_pass(window).unwrap();
    technique::bind_technique_to_render_pass(&mut techniques, &lighting_pass);
    pass::bind_device_model_to_render_pass(&mut device_model, &lighting_pass);

    while !window.handle.should_close() {
        input::update_input(window, &mut input_data);
        input::update_window_size(window);
        input::update_cursor_mode(window, &mut input_data);
        input::update_camera(&mut camera, window, &input_data);

        update_mvp_technique(
            techniques.get_mut(&technique::Techniques::MVP).unwrap(),
            &camera,
        );
        pass::resize_render_pass(&mut lighting_pass, window.width, window.height);
        pass::execute_render_pass(&lighting_pass, &techniques, &device_model);
        pass::blit_framebuffer_to_backbuffer(&lighting_pass, window);

        window.handle.swap_buffers();
    }

    pass::unbind_device_model_from_render_pass(&mut device_model, &lighting_pass);
    technique::unbind_technique_from_render_pass(&mut techniques, &lighting_pass);
    pass::delete_render_pass(&mut lighting_pass);
}

fn main() {
    let mut window = app::initialize_application();

    main_loop(&mut window);

    app::deinitialize_application();
}
