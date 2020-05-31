extern crate glfw;
use crate::app;
use crate::camera;
use crate::math;
use std::collections::HashMap;

pub type Action = glfw::Action;
pub type Key = glfw::Key;

pub struct Mouse {
    pub pos: (f64, f64),
    pub prev_pos: (f64, f64),
}

pub struct Data {
    pub keys: HashMap<Key, Action>,
    pub mouse: Mouse,
}

pub fn update_input(window: &mut app::Window, input_data: &mut Data) {
    for (_, value) in &mut input_data.keys {
        if *value == Action::Press {
            *value = Action::Repeat;
        }
    }
    input_data.keys.retain(|_, &mut v| v != Action::Release);

    window.glfw.poll_events();

    for (_, event) in glfw::flush_messages(&window.events) {
        if let glfw::WindowEvent::Key(key, _, action, _) = event {
            input_data.keys.insert(key, action);
        }
    }

    if glfw::CursorMode::Disabled == window.handle.get_cursor_mode()
        || glfw::CursorMode::Hidden == window.handle.get_cursor_mode()
    {
        input_data.mouse.prev_pos = input_data.mouse.pos;
        input_data.mouse.pos = window.handle.get_cursor_pos();
    }
}

pub fn update_window_size(window: &mut app::Window) {
    let (width, height) = window.handle.get_framebuffer_size();
    let width = width as u32;
    let height = height as u32;

    if width > 0 && height > 0 {
        if window.width != width || window.height != height {
            window.width = width;
            window.height = height;
            window.resized = true;
        } else {
            window.resized = false;
        }
    }
}

pub fn update_cursor_mode(window: &mut app::Window, input_data: &mut Data) {
    if let Some(Action::Press) = input_data.keys.get(&Key::F) {
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

pub fn update_camera(camera: &mut camera::Camera, window: &app::Window, input_data: &Data) {
    let speed: f32 = 10.;
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
