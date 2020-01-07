extern crate gl;
use crate::models;
use std::ffi::c_void;
use std::mem::size_of;

pub fn create_device_mesh(mesh: &models::HostMesh) -> models::DeviceMesh {
    let mut vao: u32 = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao as *mut u32);
        gl::BindVertexArray(vao);
    };

    assert!(
        (mesh.indices.len() % 3) == 0,
        "Index count should be multiple of 3 to render triangles.",
    );

    models::DeviceMesh {
        vao,
        index_count: mesh.indices.len() as u32 / 3,
        attributes: mesh.attributes.clone(),
        vbos: create_device_mesh_vbos(mesh),
        indices: create_buffer(&create_buffer_descriptor(
            &mesh.indices,
            gl::ELEMENT_ARRAY_BUFFER,
        ))
        .expect("Failed to create index buffer."),
        material_index: mesh.material_index,
    }
}

pub struct DeviceTextureDescriptor {
    pub s_wrap: gl::types::GLenum,
    pub t_wrap: gl::types::GLenum,
    pub mag_filter: gl::types::GLenum,
    pub min_filter: gl::types::GLenum,
    pub max_anisotropy: f32,
    pub internal_format: gl::types::GLenum,
    pub format: gl::types::GLenum,
    pub data_type: gl::types::GLenum,
}

pub fn create_color_device_texture_descriptor(
    host_texture: &Option<models::HostTexture>,
) -> Option<DeviceTextureDescriptor> {
    let mut result: Option<DeviceTextureDescriptor> = None;

    if let Some(host_texture) = host_texture {
        result = Some(DeviceTextureDescriptor {
            s_wrap: gl::REPEAT,
            t_wrap: gl::REPEAT,
            mag_filter: gl::NEAREST,
            min_filter: gl::NEAREST,
            max_anisotropy: 16_f32,
            internal_format: convert_image_depth_to_gl_internal_format(host_texture.depth),
            format: convert_image_depth_to_gl_format(host_texture.depth),
            data_type: gl::UNSIGNED_BYTE,
        });
    }

    result
}

// pub fn create_depth_device_texture_descriptor() -> DeviceTextureDescriptor {
//     DeviceTextureDescriptor {
//         s_wrap: gl::CLAMP_TO_EDGE,
//         t_wrap: gl::CLAMP_TO_EDGE,
//         mag_filter: gl::LINEAR,
//         min_filter: gl::LINEAR,
//         max_anisotropy: 1_f32,
//         internal_format: gl::DEPTH_COMPONENT32,
//         format: gl::DEPTH_COMPONENT,
//         data_type: gl::FLOAT,
//     }
// }

pub fn create_device_material(material: &models::HostMaterial) -> models::DeviceMaterial {
    models::DeviceMaterial {
        albedo_texture: create_device_texture(
            "uAlbedoMapSampler2D".to_string(),
            &material.albedo_texture,
            &create_color_device_texture_descriptor(&material.albedo_texture),
        ),
        normal_texture: create_device_texture(
            "uNormalMapSampler2D".to_string(),
            &material.normal_texture,
            &create_color_device_texture_descriptor(&material.normal_texture),
        ),
        bump_texture: create_device_texture(
            "uBumpMapSampler2D".to_string(),
            &material.bump_texture,
            &create_color_device_texture_descriptor(&material.bump_texture),
        ),
        metallic_texture: create_device_texture(
            "uMetallicSampler2D".to_string(),
            &material.metallic_texture,
            &create_color_device_texture_descriptor(&material.metallic_texture),
        ),
        roughness_texture: create_device_texture(
            "uRoughnessSampler2D".to_string(),
            &material.roughness_texture,
            &create_color_device_texture_descriptor(&material.roughness_texture),
        ),
    }
}

fn create_device_texture(
    name: String,
    host_texture: &Option<models::HostTexture>,
    desc: &Option<DeviceTextureDescriptor>,
) -> Option<models::Sampler2d> {
    let mut result: Option<models::Sampler2d> = None;

    if let (Some(host_texture), Some(desc)) = (host_texture, desc) {
        let mut handle: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut handle as *mut u32);
        }
        assert!(handle != 0, "Failed to generate texture");

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, handle);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, desc.s_wrap as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, desc.t_wrap as i32);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                desc.mag_filter as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                desc.min_filter as i32,
            );

            if desc.max_anisotropy > 1. {
                gl::TexParameterf(
                    gl::TEXTURE_2D,
                    0x84FE as gl::types::GLenum,
                    desc.max_anisotropy,
                );
            }

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                desc.internal_format as i32,
                host_texture.width as i32,
                host_texture.height as i32,
                0,
                desc.format,
                desc.data_type,
                host_texture.data.as_ptr() as *const c_void,
            )
        }

        result = Some(models::Sampler2d {
            bindings: Vec::new(),
            texture: models::DeviceTexture { name, handle },
        });
    }

    result
}

fn create_device_mesh_vbos(mesh: &models::HostMesh) -> Vec<u32> {
    let mut vbos: Vec<u32> = Vec::new();

    if !mesh.vertices.is_empty() {
        vbos.push(
            create_buffer(&create_buffer_descriptor(&mesh.vertices, gl::ARRAY_BUFFER))
                .expect("Failed to create vertex buffer."),
        );
    }

    if !mesh.normals.is_empty() {
        vbos.push(
            create_buffer(&create_buffer_descriptor(&mesh.normals, gl::ARRAY_BUFFER))
                .expect("Failed to create normal buffer."),
        );
    }

    if !mesh.uvs.is_empty() {
        vbos.push(
            create_buffer(&create_buffer_descriptor(&mesh.uvs, gl::ARRAY_BUFFER))
                .expect("Failed to create uv buffer."),
        );
    }

    vbos
}

fn convert_image_depth_to_gl_internal_format(image_depth: usize) -> gl::types::GLenum {
    match image_depth {
        1 => gl::R32F,
        2 => gl::RG32F,
        3 => gl::RGB32F,
        4 => gl::RGBA32F,
        _ => panic!(),
    }
}

fn convert_image_depth_to_gl_format(image_depth: usize) -> gl::types::GLenum {
    match image_depth {
        1 => gl::RED,
        2 => gl::RG,
        3 => gl::RGB,
        4 => gl::RGBA,
        _ => panic!(),
    }
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
