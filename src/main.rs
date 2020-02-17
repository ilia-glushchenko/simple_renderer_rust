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
fn load_skybox() -> (model::DeviceModel, math::Mat4x4f) {
    let device_model = loader::load_device_model_from_obj(Path::new("data/models/box/box.obj"));
    let transform = math::scale_mat4x4(math::Vec3f {
        x: 100.,
        y: 100.,
        z: 100.,
    });

    (device_model, transform)
}

#[allow(dead_code)]
fn load_sponza() -> (model::DeviceModel, Vec<math::Mat4x4f>) {
    let device_model =
        loader::load_device_model_from_obj(Path::new("data/models/Sponza/sponza.obj"));
    let transforms = vec![math::identity_mat4x4(); device_model.meshes.len()];

    (device_model, transforms)
}

fn create_mvp_technique(
    camera: &camera::Camera,
    transforms: &Vec<math::Mat4x4f>,
) -> technique::Technique {
    let host_cube_map_texture =
        loader::load_cube_map_texture(&Path::new("data/materials/hdri/table_mountain"));
    let texture_descriptor =
        texture::create_cube_map_device_texture_descriptor(&host_cube_map_texture);

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
                        data: vec![x],
                    })
                    .collect(),
            }],
        },
        textures: vec![model::TextureSampler {
            bindings: Vec::new(),
            texture: texture::create_device_cube_map_texture(
                "uSkyboxSamplerCube".to_string(),
                &host_cube_map_texture,
                &texture_descriptor,
            ),
        }],
    }
}

fn create_skybox_technique(
    camera: &camera::Camera,
    skybox_model: math::Mat4x4f,
) -> technique::Technique {
    let host_cube_map_texture =
        loader::load_cube_map_texture(&Path::new("data/materials/hdri/table_mountain"));
    let texture_descriptor =
        texture::create_cube_map_device_texture_descriptor(&host_cube_map_texture);

    technique::Technique {
        name: "Skybox".to_string(),
        per_frame_uniforms: technique::Uniforms {
            vec1f: Vec::new(),
            vec1u: Vec::new(),
            vec2f: Vec::new(),
            vec3f: Vec::new(),
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
                technique::Uniform::<math::Mat4x4f> {
                    name: "uModelMat4".to_string(),
                    data_location: technique::UniformDataLoction {
                        locations: Vec::new(),
                        data: vec![skybox_model],
                    },
                },
            ],
        },
        per_model_uniforms: technique::PerModelUniforms {
            vec1f: Vec::new(),
            vec1u: Vec::new(),
            vec2f: Vec::new(),
            vec3f: Vec::new(),
            mat4x4f: Vec::new(),
        },
        textures: vec![model::TextureSampler {
            bindings: Vec::new(),
            texture: texture::create_device_cube_map_texture(
                "uSkyboxSamplerCube".to_string(),
                &host_cube_map_texture,
                &texture_descriptor,
            ),
        }],
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

fn update_skybox_technique(tech: &mut technique::Technique, camera: &camera::Camera) {
    let view_mat_index = tech
        .per_frame_uniforms
        .mat4x4f
        .iter()
        .position(|x| x.name == "uViewMat4")
        .expect("Skybox technique must have uViewMat4");
    let view_mat = &mut tech.per_frame_uniforms.mat4x4f[view_mat_index]
        .data_location
        .data[0];
    *view_mat = camera.view * math::tranlation_mat4x4(camera.pos);

    let proj_mat_index = tech
        .per_frame_uniforms
        .mat4x4f
        .iter()
        .position(|x| x.name == "uProjMat4")
        .expect("Skybox technique must have uProjMat4");
    let proj_mat = &mut tech.per_frame_uniforms.mat4x4f[proj_mat_index]
        .data_location
        .data[0];
    *proj_mat =
        math::perspective_projection_mat4x4(camera.fov, camera.aspect, camera.near, camera.far);
}

fn main_loop(window: &mut app::Window) {
    let hrd_cube_map = loader::load_cube_map_texture_hdr(&Path::new(
        "data/materials/hdri/cayley_interior/cayley_interior_4k.hdr",
    ));

    let mut input_data: input::Data = input::Data {
        keys: HashMap::new(),
        mouse: input::Mouse {
            pos: (0., 0.),
            prev_pos: (0., 0.),
        },
    };

    let (mut device_model, transforms) = load_pbr_sphere();
    let mut camera = camera::create_default_camera(window.width, window.height);
    let mvp_technique = create_mvp_technique(&camera, &transforms);
    if let Err(msg) = technique::is_technique_valid(&mvp_technique) {
        log::log_error(msg);
        panic!();
    }

    let (mut skybox_model, skybox_transform) = load_skybox();
    let skybox_technique = create_skybox_technique(&camera, skybox_transform);
    if let Err(msg) = technique::is_technique_valid(&skybox_technique) {
        log::log_error(msg);
        panic!();
    }

    let mut techniques = technique::TechniqueMap::new();
    techniques.insert(technique::Techniques::MVP, mvp_technique);
    techniques.insert(technique::Techniques::Skybox, skybox_technique);

    let mut pipeline = pipeline::create_render_pipeline(&mut techniques, window);
    if let Ok(ref mut pipeline) = &mut pipeline {
        pass::bind_device_model_to_render_pass(&mut device_model, &pipeline[0]);
        pass::bind_device_model_to_render_pass(&mut device_model, &pipeline[1]);
        pass::bind_device_model_to_render_pass(&mut skybox_model, &pipeline[2]);
        if let Err(msg) = pipeline::is_render_pipeline_valid(pipeline, &techniques, &device_model) {
            log::log_error(msg);
        }
    }

    while !window.handle.should_close() {
        input::update_input(window, &mut input_data);
        input::update_window_size(window);
        input::update_cursor_mode(window, &mut input_data);
        input::update_camera(&mut camera, window, &input_data);

        update_mvp_technique(
            techniques.get_mut(&technique::Techniques::MVP).unwrap(),
            &camera,
        );
        update_skybox_technique(
            techniques.get_mut(&technique::Techniques::Skybox).unwrap(),
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
            pass::execute_render_pass(&pipeline[2], &techniques, &skybox_model);
            pass::blit_framebuffer_to_backbuffer(&pipeline.last().unwrap(), window);
        }

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
