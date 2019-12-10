extern crate colored;
extern crate gl;
extern crate gl_loader;
extern crate glfw;
use colored::*;
use glfw::Context;
use std::ffi::c_void;
use std::ffi::CStr;
use std::ptr::null;

pub struct Window {
    pub glfw: glfw::Glfw,
    pub handle: glfw::Window,
    pub events: std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,
    pub width: i32,
    pub height: i32,
}

pub fn initialize_application() -> Window {
    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let width: i32 = 1024;
    let height: i32 = 1024;

    let (mut handle, events) = glfw
        .create_window(
            width as u32,
            height as u32,
            "Simple Renderer",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    handle.make_current();
    handle.set_key_polling(true);

    gl_loader::init_gl();
    gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);

    initialize_gl_debug();

    Window {
        glfw,
        handle,
        events,
        width,
        height,
    }
}

pub fn deinitialize_application() {
    gl_loader::end_gl();
}

fn initialize_gl_debug() {
    unsafe {
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(Some(gl_debug_callback), null());
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
        println!("{}", c_str.to_str().unwrap().blue());
    } else {
        println!("{}", c_str.to_str().unwrap().red());
    }
}
