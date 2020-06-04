extern crate gl;
extern crate gl_loader;
extern crate glfw;
extern crate stb_image;

use glfw::Context;
use std::ffi::{c_void, CStr};
use std::ptr::null;

use crate::asset::model;
use crate::core::{app, camera, input, pipeline, tech};
use crate::helpers::{helper, log};
use crate::math;
use crate::ui;
use crate::{ibl, techniques};

pub struct App {
    pub glfw: glfw::Glfw,
    pub imgui: imgui::Context,
    pub imgui_glfw: ui::ImguiGLFW,
    pub window: glfw::Window,
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

        let (mut window, events) = glfw
            .create_window(width, height, "Simple Renderer", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");

        window.make_current();
        window.set_key_polling(true);
        window.set_resizable(true);
        window.set_all_polling(true);

        gl_loader::init_gl();
        gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);

        initialize_gl_debug();
        init_gl();

        //Need to flip images because of the OpenGL coordinate system
        unsafe { stb_image::stb_image::bindgen::stbi_set_flip_vertically_on_load(1) };

        let mut imgui = imgui::Context::create();
        let imgui_glfw = ui::ImguiGLFW::new(&mut imgui, &mut window);
        imgui.io_mut().mouse_draw_cursor = false;

        App {
            glfw,
            imgui,
            imgui_glfw,
            window,
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

pub fn main_loop(app: &mut app::App) {
    let mut editor = ui::editor::Editor::new();
    let mut input_data = input::Data::new();
    let mut camera = camera::create_default_camera(app.width, app.height);
    let mut model = helper::load_wall();
    let mut transforms = vec![math::scale_uniform_mat4x4(100.) * math::x_rotation_mat4x4(3.14)];

    let mut techniques = {
        let skybox_texture = ibl::create_cube_map_texture(&helper::load_hdri_texture());
        let mut techniques = tech::TechniqueContainer::new();
        techniques.map.insert(
            tech::Techniques::MVP,
            techniques::mvp::create(&camera, &transforms),
        );
        techniques.map.insert(
            tech::Techniques::Lighting,
            techniques::lighting::create(&camera, &skybox_texture),
        );
        techniques.map.insert(
            tech::Techniques::Skybox,
            techniques::skybox::create(&camera, skybox_texture, math::scale_uniform_mat4x4(500.)),
        );
        techniques.map.insert(
            tech::Techniques::ToneMapping,
            techniques::tone_mapping::create(),
        );
        for technique in techniques.map.values() {
            if let Err(msg) = tech::is_technique_valid(&technique) {
                panic!(msg);
            }
        }
        techniques
    };

    let mut pipeline = pipeline::Pipeline::new(app).unwrap();
    techniques.bind_pipeline(&pipeline);
    pipeline.bind_model(&mut model);

    if let Err(msg) = pipeline::is_render_pipeline_valid(&pipeline, &techniques, &model) {
        log::log_error(msg);
    }

    while !app.window.should_close() {
        {
            let model_load_result = editor.load_file_window.receiver.try_recv();
            if let Ok(host_model) = model_load_result {
                pipeline.unbind_model(&mut model);
                techniques.unbind_pipeline(&pipeline);

                model = model::DeviceModel::new(&host_model);
                transforms = vec![math::Mat4x4f::identity(); model.meshes.len()];
                techniques.map.insert(
                    tech::Techniques::MVP,
                    techniques::mvp::create(&camera, &transforms),
                );

                techniques.bind_pipeline(&pipeline);
                pipeline.bind_model(&mut model);
            }
        }

        input::update_input(app, &mut input_data);
        input::update_window_size(app);
        input::update_cursor_mode(app, &mut input_data);
        input::update_camera(&mut camera, app, &input_data);
        input::hot_reload(&mut pipeline, &mut techniques, &mut model, &input_data);
        input::resize(&mut pipeline, &mut techniques, &mut model, &app);

        techniques::mvp::update(
            techniques.map.get_mut(&tech::Techniques::MVP).unwrap(),
            &camera,
            &transforms,
        );
        techniques::lighting::update(
            techniques.map.get_mut(&tech::Techniques::Lighting).unwrap(),
            &camera,
        );
        techniques::skybox::update(
            techniques.map.get_mut(&tech::Techniques::Skybox).unwrap(),
            &camera,
        );

        pipeline.draw(&app, &techniques, &model);

        {
            let children_outliner_items: Vec<ui::editor::OutlinerItem> = model
                .meshes
                .iter()
                .map(|m| ui::editor::OutlinerItem {
                    label: m.name.clone(),
                    children: Vec::new(),
                })
                .collect();

            let outliner_items = vec![ui::editor::OutlinerItem {
                label: "Model".to_string(),
                children: children_outliner_items.iter().map(|x| x).collect(),
            }];

            let mut inspector_items = {
                let mut inspector_items = Vec::<ui::editor::InsepctorItem>::new();
                inspector_items.push(ui::editor::InsepctorItem {
                    label: "Transform".to_string(),
                    access: ui::editor::PropertyAccess::ReadOnly,
                    property: ui::editor::PropertyValue::Mat4x4f(&mut transforms[0]),
                });
                let material = &mut model.materials[model.meshes[0].material_index];

                for property in material.properties_1f.iter_mut() {
                    for (i, data) in property.value.data_location.data.iter_mut().enumerate() {
                        inspector_items.push(ui::editor::InsepctorItem {
                            label: format!("{0}[{1}]", property.name, i),
                            access: ui::editor::PropertyAccess::ReadWrite,
                            property: ui::editor::PropertyValue::Vec1f(data),
                        });
                    }
                }
                for property in material.properties_3f.iter_mut() {
                    for (i, data) in property.value.data_location.data.iter_mut().enumerate() {
                        inspector_items.push(ui::editor::InsepctorItem {
                            label: format!("{0}[{1}]", property.name, i),
                            access: ui::editor::PropertyAccess::ReadWrite,
                            property: ui::editor::PropertyValue::Vec3f(data),
                        });
                    }
                }

                inspector_items
            };

            let mut ui = app.imgui_glfw.frame(&mut app.window, &mut app.imgui);
            editor.draw_ui(&mut ui, &outliner_items, &mut inspector_items);

            let mut o = true;
            ui.show_demo_window(&mut o);

            app.imgui_glfw.draw(ui);
        }

        app.window.swap_buffers();
    }

    pipeline.unbind_model(&mut model);
    techniques.unbind_pipeline(&pipeline);
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
