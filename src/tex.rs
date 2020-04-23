use std::ffi::c_void;
use std::ptr::null;

#[derive(Clone)]
pub enum HostTextureData {
    UINT8(Vec<u8>),
    FLOAT32(Vec<f32>),
}

#[derive(Clone)]
pub struct HostTexture {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub data: HostTextureData,
}

#[derive(Clone)]
pub struct HostCubeMapTexture {
    pub width: usize,
    pub height: usize,
    pub nx: HostTexture,
    pub px: HostTexture,
    pub ny: HostTexture,
    pub py: HostTexture,
    pub nz: HostTexture,
    pub pz: HostTexture,
}

#[derive(Clone)]
pub struct DeviceTexture {
    pub name: String,
    pub handle: u32,
    pub target: gl::types::GLenum,
}

pub struct DeviceTextureDescriptor {
    pub target: gl::types::GLenum,
    pub s_wrap: gl::types::GLenum,
    pub t_wrap: gl::types::GLenum,
    pub r_wrap: gl::types::GLenum,
    pub mag_filter: gl::types::GLenum,
    pub min_filter: gl::types::GLenum,
    pub max_anisotropy: f32,
    pub internal_format: gl::types::GLenum,
    pub format: gl::types::GLenum,
    pub data_type: gl::types::GLenum,
    pub use_mipmaps: bool,
}

pub fn create_color_device_texture_descriptor(
    host_texture: &HostTexture,
) -> DeviceTextureDescriptor {
    DeviceTextureDescriptor {
        target: gl::TEXTURE_2D,
        s_wrap: gl::REPEAT,
        t_wrap: gl::REPEAT,
        r_wrap: gl::REPEAT,
        mag_filter: gl::LINEAR,
        min_filter: gl::LINEAR,
        max_anisotropy: 16_f32,
        internal_format: convert_image_depth_to_gl_internal_format(host_texture.depth),
        format: convert_image_depth_to_gl_format(host_texture.depth),
        data_type: gl::UNSIGNED_BYTE,
        use_mipmaps: true,
    }
}

pub fn create_depth_device_texture_descriptor() -> DeviceTextureDescriptor {
    DeviceTextureDescriptor {
        target: gl::TEXTURE_2D,
        s_wrap: gl::CLAMP_TO_EDGE,
        t_wrap: gl::CLAMP_TO_EDGE,
        r_wrap: gl::CLAMP_TO_EDGE,
        mag_filter: gl::LINEAR,
        min_filter: gl::LINEAR,
        max_anisotropy: 1_f32,
        internal_format: gl::DEPTH_COMPONENT32,
        format: gl::DEPTH_COMPONENT,
        data_type: gl::FLOAT,
        use_mipmaps: false,
    }
}

pub fn create_color_attachment_device_texture_descriptor() -> DeviceTextureDescriptor {
    DeviceTextureDescriptor {
        target: gl::TEXTURE_2D,
        s_wrap: gl::CLAMP_TO_EDGE,
        t_wrap: gl::CLAMP_TO_EDGE,
        r_wrap: gl::CLAMP_TO_EDGE,
        mag_filter: gl::LINEAR,
        min_filter: gl::LINEAR,
        max_anisotropy: 1_f32,
        internal_format: gl::RGBA32F,
        format: gl::RGBA,
        data_type: gl::FLOAT,
        use_mipmaps: false,
    }
}

pub fn create_spherical_hdri_texture_descriptor(
    host_texture: &HostTexture,
) -> DeviceTextureDescriptor {
    DeviceTextureDescriptor {
        target: gl::TEXTURE_2D,
        s_wrap: gl::CLAMP_TO_EDGE,
        t_wrap: gl::CLAMP_TO_EDGE,
        r_wrap: gl::CLAMP_TO_EDGE,
        mag_filter: gl::LINEAR,
        min_filter: gl::LINEAR,
        max_anisotropy: 1_f32,
        internal_format: convert_image_depth_to_gl_internal_format(host_texture.depth),
        format: convert_image_depth_to_gl_format(host_texture.depth),
        data_type: gl::FLOAT,
        use_mipmaps: false,
    }
}

pub fn create_empty_host_texture(
    name: String,
    width: usize,
    height: usize,
    depth: usize,
) -> HostTexture {
    HostTexture {
        name,
        width,
        height,
        depth,
        data: HostTextureData::UINT8(Vec::new()),
    }
}

pub fn create_device_texture(
    name: String,
    host_texture: &HostTexture,
    desc: &DeviceTextureDescriptor,
) -> DeviceTexture {
    let mut handle: u32 = 0;
    unsafe { gl::GenTextures(1, &mut handle as *mut u32) };
    assert!(handle != 0, "Failed to generate texture");

    unsafe {
        gl::BindTexture(desc.target, handle);
        gl::TexParameteri(desc.target, gl::TEXTURE_WRAP_S, desc.s_wrap as i32);
        gl::TexParameteri(desc.target, gl::TEXTURE_WRAP_T, desc.t_wrap as i32);
        gl::TexParameteri(desc.target, gl::TEXTURE_WRAP_R, desc.r_wrap as i32);
        gl::TexParameteri(desc.target, gl::TEXTURE_MAG_FILTER, desc.mag_filter as i32);
        gl::TexParameteri(desc.target, gl::TEXTURE_MIN_FILTER, desc.min_filter as i32);

        if desc.max_anisotropy > 1. {
            gl::TexParameterf(
                desc.target,
                0x84FE as gl::types::GLenum,
                desc.max_anisotropy,
            );
        }

        gl::TexImage2D(
            desc.target,
            0,
            desc.internal_format as i32,
            host_texture.width as i32,
            host_texture.height as i32,
            0,
            desc.format,
            desc.data_type,
            {
                match &host_texture.data {
                    HostTextureData::UINT8(data) => {
                        if data.is_empty() {
                            null()
                        } else {
                            data.as_ptr() as *const c_void
                        }
                    }
                    HostTextureData::FLOAT32(data) => {
                        if data.is_empty() {
                            null()
                        } else {
                            data.as_ptr() as *const c_void
                        }
                    }
                }
            },
        );

        if desc.use_mipmaps {
            gl::TexParameteri(
                desc.target,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(desc.target, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::GenerateMipmap(desc.target);
        }
    }

    DeviceTexture {
        name,
        handle,
        target: desc.target,
    }
}

pub fn delete_device_texture(device_texture: &DeviceTexture) {
    unsafe { gl::DeleteTextures(1, &device_texture.handle) };
}

pub fn convert_image_depth_to_gl_internal_format(image_depth: usize) -> gl::types::GLenum {
    match image_depth {
        1 => gl::R32F,
        2 => gl::RG32F,
        3 => gl::RGB32F,
        4 => gl::RGBA32F,
        _ => panic!("Unsupported image depth"),
    }
}

pub fn convert_image_depth_to_gl_format(image_depth: usize) -> gl::types::GLenum {
    match image_depth {
        1 => gl::RED,
        2 => gl::RG,
        3 => gl::RGB,
        4 => gl::RGBA,
        _ => panic!("Unsupported image depth"),
    }
}

pub fn convert_gl_format_to_image_depth(gl_format: gl::types::GLenum) -> usize {
    match gl_format {
        gl::RED | gl::DEPTH_COMPONENT => 1,
        gl::RG => 2,
        gl::RGB => 3,
        gl::RGBA => 4,
        _ => panic!("Unsupported GL format"),
    }
}
