use glfw::Context;
use std::collections::HashMap;

mod app;
mod buffer;
mod camera;
mod helper;
mod ibl;
mod input;
mod loader;
mod log;
mod material;
mod math;
mod mesh;
mod model;
mod pass;
mod pipeline;
mod shader;
mod tech;
mod techniques;
mod tex;
mod uniform;

fn main_loop(window: &mut app::Window) {
    let mut input_data: input::Data = input::Data {
        keys: HashMap::new(),
        mouse: input::Mouse {
            pos: (0., 0.),
            prev_pos: (0., 0.),
        },
    };

    let mut fullscreen_model = helper::create_full_screen_triangle_model();
    let brdf_lut = ibl::create_brdf_lut();
    let (mut box_model, specular_skybox, diffuse_skybox, env_map, skybox_transform) =
        helper::load_skybox();
    // let (mut device_model_full, transforms) = helper::load_pbr_sphere();
    // let (mut device_model_full, transforms) = helper::load_sponza();
    // let (mut device_model_full, transforms) = helper::load_bistro_exterior();
    let (mut device_model_full, transforms) = helper::load_wall();
    // let (mut device_model_full, transforms) = helper::load_cylinder();
    // let (mut device_model_full, transforms) = helper::load_well();
    let mut camera = camera::create_default_camera(window.width, window.height);

    let (mvp_technique, lighting_technique, skybox_technique, tone_mapping) = {
        let mvp = techniques::mvp::create(&camera, &transforms);
        let lighting = techniques::lighting::create(diffuse_skybox, brdf_lut, env_map, &camera);
        let skybox = techniques::skybox::create(specular_skybox, &camera, skybox_transform);
        let tone_mapping = techniques::tone_mapping::create();

        if let Err(msg) = tech::is_technique_valid(&mvp) {
            log::log_error(msg);
            panic!();
        }
        if let Err(msg) = tech::is_technique_valid(&lighting) {
            log::log_error(msg);
            panic!();
        }
        if let Err(msg) = tech::is_technique_valid(&skybox) {
            log::log_error(msg);
            panic!();
        }
        if let Err(msg) = tech::is_technique_valid(&tone_mapping) {
            log::log_error(msg);
            panic!();
        }

        (mvp, lighting, skybox, tone_mapping)
    };

    let mut techniques = tech::TechniqueContainer::new();
    techniques.map.insert(tech::Techniques::MVP, mvp_technique);
    techniques
        .map
        .insert(tech::Techniques::Lighting, lighting_technique);
    techniques
        .map
        .insert(tech::Techniques::Skybox, skybox_technique);
    techniques
        .map
        .insert(tech::Techniques::ToneMapping, tone_mapping);

    let mut pipeline = pipeline::Pipeline::new(window);
    match &mut pipeline {
        Ok(ref mut pipeline) => {
            techniques.bind_pipeline(&pipeline);
            device_model_full.bind_pass(&pipeline.passes[0]);
            device_model_full.bind_pass(&pipeline.passes[1]);
            box_model.bind_pass(&pipeline.passes[2]);
            fullscreen_model.bind_pass(&pipeline.passes[3]);
            if let Err(msg) =
                pipeline::is_render_pipeline_valid(&pipeline, &techniques, &device_model_full)
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

        techniques::mvp::update(
            techniques.map.get_mut(&tech::Techniques::MVP).unwrap(),
            &camera,
        );
        techniques::lighting::update(
            techniques.map.get_mut(&tech::Techniques::Lighting).unwrap(),
            &camera,
        );
        techniques::skybox::update(
            techniques.map.get_mut(&tech::Techniques::Skybox).unwrap(),
            &camera,
        );

        if let Ok(ref mut pipeline) = &mut pipeline {
            if let Some(input::Action::Press) = input_data.keys.get(&input::Key::F5) {
                let pipeline_program_handles: Vec<u32> =
                    pipeline.passes.iter().map(|x| x.program.handle).collect();

                if let Err(msg) = pipeline::reload_render_pipeline(pipeline) {
                    log::log_error(format!("Failed to hot reload pipeline:\n{}", msg));
                } else {
                    for pass_program_handle in pipeline_program_handles {
                        pass::unbind_techniques_from_render_pass(
                            &mut techniques,
                            pass_program_handle,
                        );
                    }

                    device_model_full.unbind_pass(pipeline.passes[0].program.handle);
                    device_model_full.unbind_pass(pipeline.passes[1].program.handle);
                    box_model.unbind_pass(pipeline.passes[2].program.handle);
                    fullscreen_model.unbind_pass(pipeline.passes[3].program.handle);

                    for pass in pipeline.passes.iter_mut() {
                        pass::bind_techniques_to_render_pass(&mut techniques, pass);
                    }

                    device_model_full.bind_pass(&pipeline.passes[0]);
                    device_model_full.bind_pass(&pipeline.passes[1]);
                    box_model.bind_pass(&pipeline.passes[2]);
                    fullscreen_model.bind_pass(&pipeline.passes[3]);

                    if let Err(msg) = pipeline::is_render_pipeline_valid(
                        pipeline,
                        &techniques,
                        &device_model_full,
                    ) {
                        log::log_error(msg);
                    }
                    log::log_info("Pipeline hot reloaded".to_string());
                }
            }

            if window.resized {
                pipeline::resize_render_pipeline(window, pipeline);
                if let Err(msg) =
                    pipeline::is_render_pipeline_valid(pipeline, &techniques, &device_model_full)
                {
                    log::log_error(msg);
                }
            }

            pass::execute_render_pass(&pipeline.passes[0], &techniques, &device_model_full);
            pass::execute_render_pass(&pipeline.passes[1], &techniques, &device_model_full);
            pass::execute_render_pass(&pipeline.passes[2], &techniques, &box_model);
            pass::execute_render_pass(&pipeline.passes[3], &techniques, &fullscreen_model);

            pass::blit_framebuffer_to_backbuffer(&pipeline.passes.last().unwrap(), window);
        }

        window.handle.swap_buffers();
    }

    if let Ok(ref mut pipeline) = pipeline {
        device_model_full.unbind_pass(pipeline.passes[0].program.handle);
        device_model_full.unbind_pass(pipeline.passes[1].program.handle);
        box_model.unbind_pass(pipeline.passes[2].program.handle);
        fullscreen_model.unbind_pass(pipeline.passes[3].program.handle);
        techniques.unbind_pipeline(&pipeline);
    }
}

fn main() {
    let mut window = app::initialize_application();

    main_loop(&mut window);

    app::deinitialize_application();
}
