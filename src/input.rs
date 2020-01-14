extern crate glfw;
use crate::window;
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

pub fn update_input(window: &mut window::Window, input_data: &mut Data) {
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
