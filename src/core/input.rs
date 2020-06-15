extern crate glfw;
use crate::asset::model;
use crate::core::{app, camera, pipeline, tech};
use crate::helpers::log;
use crate::math;
use std::collections::HashMap;

pub type Action = glfw::Action;
pub type Key = glfw::Key;

#[derive(Clone)]
pub struct Mouse {
    pub pos: (f64, f64),
    pub prev_pos: (f64, f64),
}

#[derive(Clone)]
pub struct Data {
    pub keys: HashMap<Key, Action>,
    pub mouse: Mouse,
    pub mouse_press: [bool; 5],
}

impl Data {
    pub fn new() -> Data {
        Data {
            keys: HashMap::new(),
            mouse: Mouse {
                pos: (0., 0.),
                prev_pos: (0., 0.),
            },
            mouse_press: [false; 5],
        }
    }
}

pub fn update_input(app: &mut app::App, input_data: &mut Data) {
    for (_, value) in &mut input_data.keys {
        if *value == Action::Press {
            *value = Action::Repeat;
        }
    }
    input_data.keys.retain(|_, &mut v| v != Action::Release);

    app.glfw.poll_events();

    for (_, event) in glfw::flush_messages(&app.events) {
        match event {
            glfw::WindowEvent::MouseButton(mouse_btn, action, _) => {
                let index = match mouse_btn {
                    glfw::MouseButton::Button1 => 0,
                    glfw::MouseButton::Button2 => 1,
                    glfw::MouseButton::Button3 => 2,
                    glfw::MouseButton::Button4 => 3,
                    glfw::MouseButton::Button5 => 4,
                    _ => 0,
                };
                let press = action != Action::Release;
                input_data.mouse_press[index] = press;
                app.imgui.io_mut().mouse_down = input_data.mouse_press;
            }
            glfw::WindowEvent::CursorPos(w, h) => {
                app.imgui.io_mut().mouse_pos = [w as f32, h as f32];
            }
            glfw::WindowEvent::Scroll(_, d) => {
                app.imgui.io_mut().mouse_wheel = d as f32;
            }
            glfw::WindowEvent::Char(character) => {
                app.imgui.io_mut().add_input_character(character);
            }
            glfw::WindowEvent::Key(key, _, action, modifier) => {
                app.imgui.io_mut().key_ctrl = modifier.intersects(glfw::Modifiers::Control);
                app.imgui.io_mut().key_alt = modifier.intersects(glfw::Modifiers::Alt);
                app.imgui.io_mut().key_shift = modifier.intersects(glfw::Modifiers::Shift);
                app.imgui.io_mut().key_super = modifier.intersects(glfw::Modifiers::Super);
                app.imgui.io_mut().keys_down[key as usize] = action != Action::Release;
                input_data.keys.insert(key, action);
            }
            _ => {}
        }
    }

    if glfw::CursorMode::Disabled == app.window.get_cursor_mode()
        || glfw::CursorMode::Hidden == app.window.get_cursor_mode()
    {
        input_data.mouse.prev_pos = input_data.mouse.pos;
        input_data.mouse.pos = app.window.get_cursor_pos();
    }
}

pub fn update_window_size(app: &mut app::App) {
    let (width, height) = app.window.get_framebuffer_size();
    let width = width as u32;
    let height = height as u32;

    if width > 0 && height > 0 {
        if app.width != width || app.height != height {
            app.width = width;
            app.height = height;
            app.resized = true;
        } else {
            app.resized = false;
        }
    }
}

pub fn update_cursor_mode(app: &mut app::App, input_data: &mut Data) {
    if let Some(Action::Press) = input_data.keys.get(&Key::F) {
        let mode = app.window.get_cursor_mode();
        match mode {
            glfw::CursorMode::Normal => {
                app.window.set_cursor_mode(glfw::CursorMode::Disabled);
                input_data.mouse.pos = app.window.get_cursor_pos();
                input_data.mouse.prev_pos = input_data.mouse.pos;
            }
            glfw::CursorMode::Disabled => {
                app.window.set_cursor_mode(glfw::CursorMode::Normal);
                input_data.mouse.pos = (0., 0.);
                input_data.mouse.prev_pos = input_data.mouse.pos;
            }
            _ => panic!(),
        };
    }
}

pub fn update_camera(camera: &mut camera::Camera, app: &app::App, input_data: &Data) {
    let speed: f32 = 10.;
    let world_forward = math::Vec4f::new(0., 0., -1., 0.);
    let world_right = math::Vec4f::new(-1., 0., 0., 0.);

    let mut forward = math::zero_vec4::<f32>();
    let mut right = math::zero_vec4::<f32>();
    let camera_matrix = math::create_camera_mat4x4(camera.pos, camera.yaw, camera.pitch);

    if let Some(Action::Press) | Some(Action::Repeat) = input_data.keys.get(&Key::W) {
        forward = camera_matrix * world_forward * speed;
    }
    if let Some(Action::Press) | Some(Action::Repeat) = input_data.keys.get(&Key::S) {
        forward = camera_matrix * -world_forward * speed;
    }
    if let Some(Action::Press) | Some(Action::Repeat) = input_data.keys.get(&Key::A) {
        right = camera_matrix * world_right * speed;
    }
    if let Some(Action::Press) | Some(Action::Repeat) = input_data.keys.get(&Key::D) {
        right = camera_matrix * -world_right * speed;
    }
    if let Some(Action::Press) | Some(Action::Repeat) = input_data.keys.get(&Key::Q) {
        camera.pos.y -= speed;
    }
    if let Some(Action::Press) | Some(Action::Repeat) = input_data.keys.get(&Key::E) {
        camera.pos.y += speed;
    }

    let x_dist = input_data.mouse.prev_pos.0 - input_data.mouse.pos.0;
    let y_dist = input_data.mouse.prev_pos.1 - input_data.mouse.pos.1;

    let yaw = (x_dist / app.width as f64) as f32;
    let pitch = (y_dist / app.height as f64) as f32;

    camera.pitch += pitch;
    camera.yaw += yaw;
    camera.pos.x += forward.x + right.x;
    camera.pos.y += forward.y + right.y;
    camera.pos.z += forward.z + right.z;
    camera.aspect = app.width as f32 / app.height as f32;

    camera.view = math::create_view_mat4x4(camera.pos, camera.yaw, camera.pitch);
}

pub fn hot_reload(
    pipeline: &mut pipeline::Pipeline,
    techniques: &mut tech::TechniqueContainer,
    device_model: &mut model::DeviceModel,
    input_data: &Data,
) {
    if let Some(Action::Press) = input_data.keys.get(&Key::F5) {
        pipeline.reload(techniques, device_model);
    }
}

pub fn resize(
    pipeline: &mut pipeline::Pipeline,
    techniques: &mut tech::TechniqueContainer,
    device_model: &mut model::DeviceModel,
    app: &app::App,
) {
    if app.resized {
        pipeline::resize_render_pipeline(app, pipeline);

        if let Err(msg) = pipeline::is_render_pipeline_valid(pipeline, &techniques, &device_model) {
            log::log_error(msg);
        }
    }
}
