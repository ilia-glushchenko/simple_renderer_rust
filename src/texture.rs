use std::ffi::c_void;

#[derive(Clone)]
pub struct HostTexture {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct DeviceTexture {
    pub name: String,
    pub handle: u32,
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
    host_texture: &Option<HostTexture>,
) -> Option<DeviceTextureDescriptor> {
    let mut result: Option<DeviceTextureDescriptor> = None;

    if let Some(host_texture) = host_texture {
        result = Some(DeviceTextureDescriptor {
            s_wrap: gl::REPEAT,
            t_wrap: gl::REPEAT,
            mag_filter: gl::LINEAR,
            min_filter: gl::LINEAR,
            max_anisotropy: 16_f32,
            internal_format: convert_image_depth_to_gl_internal_format(host_texture.depth),
            format: convert_image_depth_to_gl_format(host_texture.depth),
            data_type: gl::UNSIGNED_BYTE,
        });
    }

    result
}

pub fn create_device_texture(
    name: String,
    host_texture: &HostTexture,
    desc: &DeviceTextureDescriptor,
) -> DeviceTexture {
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
        );

        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

    DeviceTexture { name, handle }
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
