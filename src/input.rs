extern crate glfw;
use crate::window;
use colored::*;
use std::collections::HashMap;

pub type Action = glfw::Action;
pub type Key = glfw::Key;

pub struct Data {
    pub keys: HashMap<Key, Action>,
}

pub fn update_input(window: &mut window::Window, input_data: &mut Data) {
    window.glfw.poll_events();

    for (_, event) in glfw::flush_messages(&window.events) {
        if let glfw::WindowEvent::Key(key, _, action, _) = event {
            let message = format!("{:?}", event).to_string();
            println!("{}", message.blue());

            input_data.keys.insert(key, action);
        }
    }
}
