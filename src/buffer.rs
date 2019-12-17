extern crate gl;
use crate::models;
use std::ffi::c_void;
use std::mem::size_of;

pub fn create_device_model(model: &models::HostModel) -> models::DeviceModel {
    let mut vao: u32 = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao as *mut u32);
        gl::BindVertexArray(vao);
    };

    assert!(
        (model.indices.len() % 3) == 0,
        "Index count should be multiple of 3 to render triangles.",
    );

    models::DeviceModel {
        vao,
        index_count: model.indices.len() as i32 / 3,
        attributes: model.attributes.clone(),
        vbos: create_device_model_vbos(model),
        indices: create_buffer(&create_buffer_descriptor(
            &model.indices,
            gl::ELEMENT_ARRAY_BUFFER,
        ))
        .expect("Failed to create index buffer."),
    }
}

fn create_device_model_vbos(model: &models::HostModel) -> Vec<u32> {
    let mut vbos: Vec<u32> = Vec::new();

    if !model.vertices.is_empty() {
        vbos.push(
            create_buffer(&create_buffer_descriptor(&model.vertices, gl::ARRAY_BUFFER))
                .expect("Failed to create vertex buffer."),
        );
    }

    if !model.normals.is_empty() {
        vbos.push(
            create_buffer(&create_buffer_descriptor(&model.normals, gl::ARRAY_BUFFER))
                .expect("Failed to create normal buffer."),
        );
    }

    if !model.uvs.is_empty() {
        vbos.push(
            create_buffer(&create_buffer_descriptor(&model.uvs, gl::ARRAY_BUFFER))
                .expect("Failed to create uv buffer."),
        );
    }

    if !model.materials.is_empty() {
        vbos.push(
            create_buffer(&create_buffer_descriptor(
                &model.materials,
                gl::ARRAY_BUFFER,
            ))
            .expect("Failed to create material buffer."),
        );
    }

    vbos
}

struct BufferDescriptor {
    target: u32,
    size: u32,
    count: u32,
    data: *const c_void,
}

#[allow(clippy::ptr_arg)]
fn create_buffer_descriptor<T>(vector: &Vec<T>, target: u32) -> BufferDescriptor {
    BufferDescriptor {
        target,
        size: size_of::<T>() as u32,
        count: vector.len() as u32,
        data: vector.as_ptr() as *const c_void,
    }
}

fn create_buffer(desc: &BufferDescriptor) -> Result<u32, String> {
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
