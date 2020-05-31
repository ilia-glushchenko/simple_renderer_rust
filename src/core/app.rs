extern crate gl;
extern crate gl_loader;
extern crate glfw;
extern crate stb_image;

use glfw::Context;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ffi::CStr;
use std::ptr::null;

use crate::core::{app, camera, input, pass, pipeline, tech};
use crate::helpers::{helper, log};
use crate::{ibl, techniques};

pub struct App {
    pub glfw: glfw::Glfw,
    pub handle: glfw::Window,
    pub events: std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,
    pub width: u32,
    pub height: u32,
    pub resized: bool,
}

impl App {
    pub fn new() -> App {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        let width: u32 = 1024;
        let height: u32 = 1024;

        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
        glfw.window_hint(glfw::WindowHint::Resizable(true));
        glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));

        let (mut handle, events) = glfw
            .create_window(width, height, "Simple Renderer", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");

        handle.make_current();
        handle.set_key_polling(true);
        handle.set_resizable(true);

        gl_loader::init_gl();
        gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);

        initialize_gl_debug();
        init_gl();

        //Need to flip images because of the OpenGL coordinate system
        unsafe {
            stb_image::stb_image::bindgen::stbi_set_flip_vertically_on_load(1);
        }

        App {
            glfw,
            handle,
            events,
            width,
            height,
            resized: false,
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        gl_loader::end_gl();
    }
}

pub fn main_loop(window: &mut app::App) {
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

fn init_gl() {
    unsafe { gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS) };
}

fn initialize_gl_debug() {
    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(Some(gl_debug_callback), null());
        gl::DebugMessageControl(
            gl::DONT_CARE,
            gl::DONT_CARE,
            gl::DONT_CARE,
            0,
            null(),
            gl::TRUE,
        );
    }
}

extern "system" fn gl_debug_callback(
    _source: u32,
    gltype: u32,
    _id: u32,
    _severity: u32,
    _length: i32,
    message: *const i8,
    _user_param: *mut c_void,
) {
    let c_str: &CStr = unsafe { CStr::from_ptr(message) };
    if gltype == gl::DEBUG_TYPE_OTHER || gltype == gl::DEBUG_TYPE_MARKER {
        //log::log_info(c_str.to_str().unwrap().to_string());
    } else {
        log::log_error(c_str.to_str().unwrap().to_string());
    }
}
