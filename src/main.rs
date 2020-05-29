use glfw::Context;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

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
fn create_empty_model() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
    return (
        model::DeviceModel {
            meshes: Vec::new(),
            materials: Vec::new(),
        },
        Vec::new(),
    );
}

#[allow(dead_code)]
fn load_wall() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
    let mut device_model =
        loader::load_device_model_from_obj(Path::new("data/models/quad/quad.obj"));
    device_model.materials[0]
        .set_svec1f("uScalarRoughnessVec1f", math::Vec1f::new(0.))
        .unwrap();
    device_model.materials[0]
        .set_svec1f("uScalarMetalnessVec1f", math::Vec1f::new(0.))
        .unwrap();
    device_model.materials[0]
        .set_svec3f("uScalarAlbedoVec3f", math::Vec3f::new(0., 0., 0.))
        .unwrap();
    let trasforms = vec![
        math::tranlation_mat4x4(math::Vec3f::new(0., 0., 0.))
            * math::scale_mat4x4(math::Vec3f::new(100., 100., 100.)),
    ];

    (device_model, trasforms)
}

// #[allow(dead_code)]
// fn load_pbr_sphere() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
//     let mut device_model =
//         loader::load_device_model_from_obj(Path::new("data/models/pbr-sphere/sphere.obj"));

//     let mesh = device_model.meshes[0].clone();
//     let mut material = device_model.materials[0].clone();
//     material
//         .set_svec3f("uScalarAlbedoVec3f", math::Vec3f::new(1., 1., 1.))
//         .unwrap();
//     material.properties_3f[0].value.data_location.data[0] = math::Vec3f::new(1., 1., 1.);
//     let mut transforms = Vec::new();

//     device_model.meshes.clear();
//     device_model.materials.clear();

//     for x in (-10..12).step_by(2) {
//         for y in (-10..12).step_by(2) {
//             let mut material = material.clone();
//             let roughness = (x + 10) as f32 / 20.;
//             let metalness = (y + 10) as f32 / 20.;

//             material
//                 .set_svec1f("uScalarRoughnessVec1f", math::Vec1f::new(roughness))
//                 .unwrap();
//             material
//                 .set_svec1f("uScalarMetalnessVec1f", math::Vec1f::new(metalness))
//                 .unwrap();
//             device_model.materials.push(material);

//             let mut mesh = mesh.clone();
//             mesh.material_index = device_model.materials.len() - 1;
//             device_model.meshes.push(mesh.clone());

//             transforms.push(math::tranlation_mat4x4(math::Vec3f {
//                 x: x as f32,
//                 y: y as f32,
//                 z: 0.,
//             }));
//         }
//     }

//     (device_model, transforms)
// }

#[allow(dead_code)]
fn load_pbr_spheres() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
    let device_model =
        loader::load_device_model_from_obj(Path::new("data/models/pbr-spheres/spheres.obj"));
    let transforms = vec![
        math::tranlation_mat4x4(math::Vec3f::new(-25., -25., -65.));
        device_model.meshes.len()
    ];

    (device_model, transforms)
}

#[allow(dead_code)]
fn load_skybox() -> (
    model::DeviceModel,
    Rc<tex::DeviceTexture>,
    Rc<tex::DeviceTexture>,
    Rc<tex::DeviceTexture>,
    math::Mat4x4f,
) {
    let mut box_model = loader::load_device_model_from_obj(Path::new("data/models/box/box.obj"));
    box_model.materials = vec![model::DeviceMaterial::empty()];
    let transform = math::scale_mat4x4(math::Vec3f::new(500., 500., 500.));

    let hdr_skybox_texture = ibl::create_specular_cube_map_texture(
        &loader::load_host_texture_from_file(
            &Path::new("data/materials/hdri/quattro_canti/quattro_canti_8k.hdr"),
            "uHdriSampler2D",
        )
        .unwrap(),
        &mut box_model,
    );

    let hdr_diffuse_skybox =
        ibl::create_diffuse_cube_map_texture(hdr_skybox_texture.clone(), &mut box_model);

    let prefiltered_env_map =
        ibl::create_prefiltered_environment_map(hdr_skybox_texture.clone(), &mut box_model);

    (
        box_model,
        hdr_skybox_texture,
        hdr_diffuse_skybox,
        prefiltered_env_map,
        transform,
    )
}

#[allow(dead_code)]
fn load_sponza() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
    let device_model =
        loader::load_device_model_from_obj(Path::new("data/models/Sponza/sponza.obj"));
    let transforms = vec![math::identity_mat4x4(); device_model.meshes.len()];

    (device_model, transforms)
}

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
    let (mut box_model, specular_skybox, diffuse_skybox, env_map, skybox_transform) = load_skybox();
    // let (mut device_model_full, transforms) = load_pbr_sphere();
    // let (mut device_model_full, transforms) = load_sponza();
    let (mut device_model_full, transforms) = load_wall();
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
            pass::bind_device_model_to_render_pass(&mut device_model_full, &pipeline.passes[0]);
            pass::bind_device_model_to_render_pass(&mut device_model_full, &pipeline.passes[1]);
            pass::bind_device_model_to_render_pass(&mut box_model, &pipeline.passes[2]);
            pass::bind_device_model_to_render_pass(&mut fullscreen_model, &pipeline.passes[3]);
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

                    pass::unbind_device_model_from_render_pass(
                        &mut device_model_full,
                        pipeline.passes[0].program.handle,
                    );
                    pass::unbind_device_model_from_render_pass(
                        &mut device_model_full,
                        pipeline.passes[1].program.handle,
                    );
                    pass::unbind_device_model_from_render_pass(
                        &mut box_model,
                        pipeline.passes[2].program.handle,
                    );
                    pass::unbind_device_model_from_render_pass(
                        &mut fullscreen_model,
                        pipeline.passes[3].program.handle,
                    );

                    for pass in pipeline.passes.iter_mut() {
                        pass::bind_techniques_to_render_pass(&mut techniques, pass);
                    }

                    pass::bind_device_model_to_render_pass(
                        &mut device_model_full,
                        &pipeline.passes[0],
                    );
                    pass::bind_device_model_to_render_pass(
                        &mut device_model_full,
                        &pipeline.passes[1],
                    );
                    pass::bind_device_model_to_render_pass(&mut box_model, &pipeline.passes[2]);
                    pass::bind_device_model_to_render_pass(
                        &mut fullscreen_model,
                        &pipeline.passes[3],
                    );

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
        pass::unbind_device_model_from_render_pass(
            &mut device_model_full,
            pipeline.passes[0].program.handle,
        );
        pass::unbind_device_model_from_render_pass(
            &mut device_model_full,
            pipeline.passes[1].program.handle,
        );
        pass::unbind_device_model_from_render_pass(
            &mut box_model,
            pipeline.passes[2].program.handle,
        );
        pass::unbind_device_model_from_render_pass(
            &mut fullscreen_model,
            pipeline.passes[3].program.handle,
        );
        techniques.unbind_pipeline(&pipeline);
    }
}

fn main() {
    let mut window = app::initialize_application();

    main_loop(&mut window);

    app::deinitialize_application();
}
