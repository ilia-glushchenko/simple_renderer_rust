extern crate gl;
use std::ffi::c_void;
use std::mem::size_of;

pub struct BufferDescriptor {
    target: u32,
    size: u32,
    count: u32,
    data: *const c_void,
}

pub fn create_buffer(desc: &BufferDescriptor) -> Result<u32, String> {
    assert!(desc.size > 0);
    assert!(desc.count > 0);
    assert!(!desc.data.is_null());

    let mut handle: u32 = 0;

    unsafe {
        gl::GenBuffers(1, &mut handle as *mut u32);
    }

    if handle == 0 {
        return Result::Err("Failed to generate buffer.".to_string());
    }

    unsafe {
        gl::BindBuffer(desc.target, handle);
        gl::BufferData(
            desc.target,
            (desc.size * desc.count) as isize,
            desc.data,
            gl::STATIC_DRAW,
        );
    }

    Result::Ok(handle)
}

pub fn create_buffer_descriptor<T>(vector: &[T], target: u32) -> BufferDescriptor {
    BufferDescriptor {
        target,
        size: size_of::<T>() as u32,
        count: vector.len() as u32,
        data: vector.as_ptr() as *const c_void,
    }
}
