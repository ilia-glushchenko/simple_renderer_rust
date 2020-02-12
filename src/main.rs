use glfw::Context;
//use imgui;
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
mod pipeline;
mod shader;
mod technique;
mod texture;

fn create_mvp_technique(
    camera: &camera::Camera,
    transforms: &Vec<math::Mat4x4f>,
) -> technique::Technique {
    technique::Technique {
        name: "MVP".to_string(),
        per_frame_uniforms: technique::Uniforms {
            vec1f: Vec::new(),
            vec1u: Vec::new(),
            vec2f: Vec::new(),
            vec3f: vec![technique::Uniform::<math::Vec3f> {
                name: "uCameraPosVec3".to_string(),
                data_location: technique::UniformDataLoction {
                    locations: Vec::new(),
                    data: vec![camera.pos],
                },
            }],
            mat4x4f: vec![
                technique::Uniform::<math::Mat4x4f> {
                    name: "uProjMat4".to_string(),
                    data_location: technique::UniformDataLoction {
                        locations: Vec::new(),
                        data: vec![math::perspective_projection_mat4x4(
                            camera.fov,
                            camera.aspect,
                            camera.near,
                            camera.far,
                        )],
                    },
                },
                technique::Uniform::<math::Mat4x4f> {
                    name: "uViewMat4".to_string(),
                    data_location: technique::UniformDataLoction {
                        locations: Vec::new(),
                        data: vec![math::tranlation_mat4x4(math::Vec3f {
                            x: 0.,
                            y: 0.,
                            z: -1.,
                        })],
                    },
                },
            ],
        },
        per_model_uniforms: technique::PerModelUniforms {
            vec1f: Vec::new(),
            vec1u: Vec::new(),
            vec2f: Vec::new(),
            vec3f: Vec::new(),
            mat4x4f: vec![technique::PerModelUnifrom::<math::Mat4x4f> {
                name: "uModelMat4".to_string(),
                data_locations: transforms
                    .iter()
                    .map(|&x| technique::UniformDataLoction::<math::Mat4x4f> {
                        locations: Vec::new(),
                        data: vec![x * math::scale_uniform_mat4x4(5.)],
                    })
                    .collect(),
            }],
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
    let view_mat = &mut tech.per_frame_uniforms.mat4x4f[view_mat_index]
        .data_location
        .data[0];
    *view_mat = camera.view;

    let proj_mat_index = tech
        .per_frame_uniforms
        .mat4x4f
        .iter()
        .position(|x| x.name == "uProjMat4")
        .expect("MVP technique must have uProjMat4");
    let proj_mat = &mut tech.per_frame_uniforms.mat4x4f[proj_mat_index]
        .data_location
        .data[0];
    *proj_mat =
        math::perspective_projection_mat4x4(camera.fov, camera.aspect, camera.near, camera.far);

    let camera_pos_index = tech
        .per_frame_uniforms
        .vec3f
        .iter()
        .position(|x| x.name == "uCameraPosVec3")
        .expect("MVP technique must have uCameraPosVec3");
    let camera_pos_vec = &mut tech.per_frame_uniforms.vec3f[camera_pos_index]
        .data_location
        .data[0];
    *camera_pos_vec = camera.pos;
}

fn main_loop(window: &mut app::Window) {
    let mut input_data: input::Data = input::Data {
        keys: HashMap::new(),
        mouse: input::Mouse {
            pos: (0., 0.),
            prev_pos: (0., 0.),
        },
    };

    let mut device_model =
        loader::load_device_model_from_obj(Path::new("data/models/sponza/sponza.obj"));
    let transforms = vec![math::identity_mat4x4(); device_model.meshes.len()];
    let mut camera = camera::create_default_camera(window.width, window.height);
    let mvp_technique = create_mvp_technique(&camera, &transforms);
    if let Err(msg) = technique::is_technique_valid(&mvp_technique) {
        log::log_error(msg.clone());
        panic!();
    }
    let mut techniques = technique::TechniqueMap::new();
    techniques.insert(technique::Techniques::MVP, mvp_technique);

    let mut pipeline = pipeline::create_render_pipeline(&mut techniques, window);
    if let Ok(ref mut pipeline) = &mut pipeline {
        pass::bind_device_model_to_render_pass(&mut device_model, &pipeline[0]);
        pass::bind_device_model_to_render_pass(&mut device_model, &pipeline[1]);
        if let Err(msg) = pipeline::is_render_pipeline_valid(pipeline, &techniques, &device_model) {
            log::log_error(msg);
        }
    }

    //let mut imgui_context = imgui::Context::create();

    while !window.handle.should_close() {
        input::update_input(window, &mut input_data);
        input::update_window_size(window);
        input::update_cursor_mode(window, &mut input_data);
        input::update_camera(&mut camera, window, &input_data);

        update_mvp_technique(
            techniques.get_mut(&technique::Techniques::MVP).unwrap(),
            &camera,
        );

        if let Ok(ref mut pipeline) = &mut pipeline {
            if let Some(input::Action::Press) = input_data.keys.get(&input::Key::F5) {
                let pipeline_program_handles: Vec<u32> =
                    pipeline.iter().map(|x| x.program.handle).collect();

                if let Err(msg) = pipeline::reload_render_pipeline(pipeline) {
                    log::log_error(format!("Failed to hot reload pipeline:\n{}", msg));
                } else {
                    for pass_program_handle in pipeline_program_handles {
                        pass::unbind_technique_from_render_pass(
                            &mut techniques,
                            pass_program_handle,
                        );
                        pass::unbind_device_model_from_render_pass(
                            &mut device_model,
                            pass_program_handle,
                        );
                    }

                    for pass in pipeline.iter_mut() {
                        pass::bind_device_model_to_render_pass(&mut device_model, &pass);
                        pass::bind_technique_to_render_pass(&mut techniques, pass);
                    }

                    log::log_info("Pipeline hot reloaded".to_string());

                    if let Err(msg) =
                        pipeline::is_render_pipeline_valid(pipeline, &techniques, &device_model)
                    {
                        log::log_error(msg);
                    }
                }
            }

            if window.resized {
                pipeline::resize_render_pipeline(window, pipeline);
            }

            pass::execute_render_pass(&pipeline[0], &techniques, &device_model);
            pass::execute_render_pass(&pipeline[1], &techniques, &device_model);
            pass::blit_framebuffer_to_backbuffer(&pipeline.last().unwrap(), window);
        }

        // let mut ui = imgui_context.frame();
        // let imgui_window = imgui::Window::new(imgui::im_str!("Test window"))
        //     .size([300.0, 100.0], imgui::Condition::FirstUseEver)
        //     .build(&ui, || {
        //         ui.text(imgui::im_str!("Hello world!"));
        //         ui.text(imgui::im_str!("こんにちは世界！"));
        //         ui.text(imgui::im_str!("This...is...imgui-rs!"));
        //         ui.separator();
        //         let mouse_pos = ui.io().mouse_pos;
        //         ui.text(format!(
        //             "Mouse Position: ({:.1},{:.1})",
        //             mouse_pos[0], mouse_pos[1]
        //         ));
        //     });
        // ui.render();

        window.handle.swap_buffers();
    }

    if let Ok(ref mut pipeline) = pipeline {
        pass::unbind_device_model_from_render_pass(&mut device_model, pipeline[0].program.handle);
        pass::unbind_device_model_from_render_pass(&mut device_model, pipeline[1].program.handle);
        pipeline::delete_render_pipeline(&mut techniques, pipeline);
    }
}

fn main() {
    let mut window = app::initialize_application();

    main_loop(&mut window);

    app::deinitialize_application();
}
