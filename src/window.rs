extern crate colored;
extern crate gl;
extern crate gl_loader;
extern crate glfw;
extern crate stb_image;
use colored::*;
use glfw::Context;
use std::ffi::c_void;
use std::ffi::CStr;
use std::ptr::null;

pub struct Window {
    pub glfw: glfw::Glfw,
    pub handle: glfw::Window,
    pub events: std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,
    pub width: u32,
    pub height: u32,
}

pub fn initialize_application() -> Window {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    let width: u32 = 2048;
    let height: u32 = 2048;

    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::Resizable(true));

    let (mut handle, events) = glfw
        .create_window(width, height, "Simple Renderer", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    handle.make_current();
    handle.set_key_polling(true);
    handle.set_resizable(true);

    gl_loader::init_gl();
    gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);

    initialize_gl_debug();

    //Need to flip images because of the OpenGL coordinate system
    unsafe {
        stb_image::stb_image::bindgen::stbi_set_flip_vertically_on_load(1);
    }

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
