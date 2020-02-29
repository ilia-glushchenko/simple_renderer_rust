use glfw::Context;
use std::collections::HashMap;
use std::path::Path;

mod app;
mod buffer;
mod camera;
mod helper;
mod ibl;
mod input;
mod loader;
mod log;
mod math;
mod model;
mod pass;
mod pipeline;
mod shader;
mod tech;
mod techniques;
mod tex;

#[allow(dead_code)]
fn load_pbr_sphere() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
    let mut device_model =
        loader::load_device_model_from_obj(Path::new("data/models/pbr-sphere/sphere.obj"));
    let transforms = vec![
        math::tranlation_mat4x4(math::Vec3f {
            x: 0.,
            y: 0.,
            z: -1.5,
        });
        device_model.meshes.len()
    ];

    device_model.materials[0].scalar_albedo.data_location.data[0] = math::Vec3f {
        x: 1.,
        y: 1.,
        z: 1.,
    };
    device_model.materials[0]
        .scalar_roughness
        .data_location
        .data[0] = math::Vec1f { x: 0.95 };

    device_model.materials[0]
        .scalar_metalness
        .data_location
        .data[0] = math::Vec1f { x: 0.05 };

    (device_model, transforms)
}

#[allow(dead_code)]
fn load_pbr_spheres() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
    let device_model =
        loader::load_device_model_from_obj(Path::new("data/models/pbr-spheres/spheres.obj"));
    let transforms = vec![
        math::tranlation_mat4x4(math::Vec3f {
            x: -25.,
            y: -25.,
            z: -65.
        });
        device_model.meshes.len()
    ];

    (device_model, transforms)
}

#[allow(dead_code)]
fn load_skybox() -> (model::DeviceModel, tex::DeviceTexture, math::Mat4x4f) {
    let device_model = loader::load_device_model_from_obj(Path::new("data/models/box/box.obj"));
    let transform = math::scale_mat4x4(math::Vec3f {
        x: 100.,
        y: 100.,
        z: 100.,
    });

    let hdr_skybox_texture = ibl::create_specular_cube_map_texture(
        &loader::load_host_texture_from_file(&Path::new(
            "data/materials/hdri/venice_sunset/venice_sunset_8k.hdr",
        ))
        .unwrap(),
    );

    (device_model, hdr_skybox_texture, transform)
}

#[allow(dead_code)]
fn load_sponza() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
    let device_model =
        loader::load_device_model_from_obj(Path::new("data/models/Sponza/sponza.obj"));
    let transforms = vec![math::identity_mat4x4(); device_model.meshes.len()];

    (device_model, transforms)
}

fn load_full_screen_triangle_model() -> model::DeviceModel {
    model::create_device_model(&model::HostModel {
        meshes: vec![helper::create_full_screen_triangle_host_mesh()],
        materials: vec![model::create_empty_host_material()],
    })
}

fn main_loop(window: &mut app::Window) {
    let mut input_data: input::Data = input::Data {
        keys: HashMap::new(),
        mouse: input::Mouse {
            pos: (0., 0.),
            prev_pos: (0., 0.),
        },
    };

    let mut fullscreen_model = load_full_screen_triangle_model();
    let (mut skybox_model, hdr_skybox_texture, skybox_transform) = load_skybox();
    let (mut device_model, transforms) = load_pbr_sphere();
    let mut camera = camera::create_default_camera(window.width, window.height);

    let mvp_technique = techniques::mvp::create(hdr_skybox_texture.clone(), &camera, &transforms);
    if let Err(msg) = tech::is_technique_valid(&mvp_technique) {
        log::log_error(msg);
        panic!();
    }

    let skybox_technique =
        techniques::skybox::create(hdr_skybox_texture.clone(), &camera, skybox_transform);
    if let Err(msg) = tech::is_technique_valid(&skybox_technique) {
        log::log_error(msg);
        panic!();
    }

    let tone_mapping = techniques::tone_mapping::create();

    let mut techniques = tech::TechniqueMap::new();
    techniques.insert(tech::Techniques::MVP, mvp_technique);
    techniques.insert(tech::Techniques::Skybox, skybox_technique);
    techniques.insert(tech::Techniques::ToneMapping, tone_mapping);

    let mut pipeline = pipeline::create_render_pipeline(&mut techniques, window);
    match &mut pipeline {
        Ok(ref mut pipeline) => {
            pass::bind_device_model_to_render_pass(&mut device_model, &pipeline[0]);
            pass::bind_device_model_to_render_pass(&mut device_model, &pipeline[1]);
            pass::bind_device_model_to_render_pass(&mut skybox_model, &pipeline[2]);
            pass::bind_device_model_to_render_pass(&mut fullscreen_model, &pipeline[3]);
            if let Err(msg) =
                pipeline::is_render_pipeline_valid(pipeline, &techniques, &device_model)
            {
                log::log_error(msg);
            }
        }
        Err(msg) => {
            log::log_error(msg.clone());
            panic!();
        }
    }

    while !window.handle.should_close() {
        input::update_input(window, &mut input_data);
        input::update_window_size(window);
        input::update_cursor_mode(window, &mut input_data);
        input::update_camera(&mut camera, window, &input_data);

        techniques::mvp::update(techniques.get_mut(&tech::Techniques::MVP).unwrap(), &camera);
        techniques::skybox::update(
            techniques.get_mut(&tech::Techniques::Skybox).unwrap(),
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
                    }

                    pass::unbind_device_model_from_render_pass(
                        &mut device_model,
                        pipeline[0].program.handle,
                    );
                    pass::unbind_device_model_from_render_pass(
                        &mut device_model,
                        pipeline[1].program.handle,
                    );
                    pass::unbind_device_model_from_render_pass(
                        &mut skybox_model,
                        pipeline[2].program.handle,
                    );
                    pass::unbind_device_model_from_render_pass(
                        &mut fullscreen_model,
                        pipeline[3].program.handle,
                    );

                    for pass in pipeline.iter_mut() {
                        pass::bind_technique_to_render_pass(&mut techniques, pass);
                    }

                    pass::bind_device_model_to_render_pass(&mut device_model, &pipeline[0]);
                    pass::bind_device_model_to_render_pass(&mut device_model, &pipeline[1]);
                    pass::bind_device_model_to_render_pass(&mut skybox_model, &pipeline[2]);
                    pass::bind_device_model_to_render_pass(&mut fullscreen_model, &pipeline[3]);

                    if let Err(msg) =
                        pipeline::is_render_pipeline_valid(pipeline, &techniques, &device_model)
                    {
                        log::log_error(msg);
                    }
                    log::log_info("Pipeline hot reloaded".to_string());
                }
            }

            if window.resized {
                pipeline::resize_render_pipeline(window, pipeline);
                if let Err(msg) =
                    pipeline::is_render_pipeline_valid(pipeline, &techniques, &device_model)
                {
                    log::log_error(msg);
                }
            }

            pass::execute_render_pass(&pipeline[0], &techniques, &device_model);
            pass::execute_render_pass(&pipeline[1], &techniques, &device_model);
            pass::execute_render_pass(&pipeline[2], &techniques, &skybox_model);
            pass::execute_render_pass(&pipeline[3], &techniques, &fullscreen_model);

            pass::blit_framebuffer_to_backbuffer(&pipeline.last().unwrap(), window);
        }

        window.handle.swap_buffers();
    }

    if let Ok(ref mut pipeline) = pipeline {
        pass::unbind_device_model_from_render_pass(&mut device_model, pipeline[0].program.handle);
        pass::unbind_device_model_from_render_pass(&mut device_model, pipeline[1].program.handle);
        pass::unbind_device_model_from_render_pass(&mut skybox_model, pipeline[2].program.handle);
        pass::unbind_device_model_from_render_pass(
            &mut fullscreen_model,
            pipeline[3].program.handle,
        );
        pipeline::delete_render_pipeline(&mut techniques, pipeline);
    }

    tech::delete_techniques(techniques);
    model::delete_device_model(device_model);
    model::delete_device_model(skybox_model);
    model::delete_device_model(fullscreen_model);
}

fn main() {
    let mut window = app::initialize_application();

    main_loop(&mut window);

    app::deinitialize_application();
}
