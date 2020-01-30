extern crate gl;
use crate::app;
use crate::loader;
use crate::log;
use crate::math;
use crate::model;
use crate::shader;
use crate::technique;
use crate::texture;
use std::collections::HashMap;
use std::ptr::null;

#[derive(Copy, Clone)]
pub enum PassAttachmentType {
    Color(math::Vec4f),
    Depth(f32, gl::types::GLenum),
}

#[derive(Clone)]
pub struct RemotePassAttachmentSource {
    pub pipeline_index: usize,
    pub attachment_index: usize,
    pub device_texture: texture::DeviceTexture,
}

#[derive(Clone)]
pub enum PassAttachmentSource {
    Local,
    Remote(RemotePassAttachmentSource),
}

#[derive(Clone)]
pub struct PassAttachmentDescriptor {
    pub flavor: PassAttachmentType,
    pub source: PassAttachmentSource,
    pub write: bool,
    pub clear: bool,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone)]
pub struct PassAttachment {
    pub texture: texture::DeviceTexture,
    pub desc: PassAttachmentDescriptor,
}

pub struct PassDescriptor {
    pub name: String,
    pub program: shader::HostShaderProgramDescriptor,
    pub techniques: Vec<technique::Techniques>,

    pub attachments: Vec<PassAttachmentDescriptor>,
    pub dependencies: Vec<model::Sampler2d>,

    pub width: u32,
    pub height: u32,
}

#[derive(Clone)]
pub struct Pass {
    pub name: String,
    pub program: shader::ShaderProgram,
    pub techniques: Vec<technique::Techniques>,

    pub attachments: Vec<PassAttachment>,
    pub dependencies: Vec<model::Sampler2d>,

    pub fbo: u32,
    pub width: u32,
    pub height: u32,
}

pub fn create_render_pass(mut desc: PassDescriptor) -> Result<Pass, String> {
    let host_program = loader::load_host_shader_program(&desc.program);
    if let Result::Err(msg) = host_program {
        return Result::Err(msg);
    }
    let host_program = host_program.unwrap();

    let device_program = shader::create_shader_program(&host_program);
    if let Result::Err(msg) = device_program {
        return Result::Err(msg);
    }
    let device_program = device_program.unwrap();

    let attachments = create_pass_attachments(&desc.attachments);
    let framebuffer_object = create_framebuffer_object(&attachments);
    if let Result::Err(msg) = framebuffer_object {
        return Result::Err(msg);
    }

    for dependency in &mut desc.dependencies {
        model::bind_shader_program_to_texture(&device_program, dependency);
    }

    Result::Ok(Pass {
        name: desc.name,
        program: device_program,
        techniques: desc.techniques,
        attachments,
        dependencies: desc.dependencies,
        fbo: framebuffer_object.unwrap(),
        width: desc.width,
        height: desc.height,
    })
}

pub fn delete_render_pass(pass: &mut Pass) {
    shader::delete_shader_program(&mut pass.program);
    delete_pass_attachments(&mut pass.attachments);
    unsafe { gl::DeleteFramebuffers(1, &pass.fbo as *const u32) };
    pass.dependencies.clear();
}

pub fn bind_device_model_to_render_pass(device_model: &mut model::DeviceModel, pass: &Pass) {
    for device_mesh in &mut device_model.meshes {
        model::bind_device_mesh_to_shader_program(device_mesh, &pass.program);
    }
    for device_material in &mut device_model.materials {
        model::bind_shader_program_to_material(device_material, &pass.program);
    }
}

pub fn unbind_device_model_from_render_pass(device_model: &mut model::DeviceModel, pass: &Pass) {
    for device_material in &mut device_model.materials {
        model::unbind_shader_program_from_material(device_material, &pass.program);
    }
}

pub fn bind_technique_to_render_pass(techniques: &mut technique::TechniqueMap, pass: &Pass) {
    for (_, technique) in techniques.iter_mut() {
        technique::bind_shader_program_to_technique(technique, &pass.program);
    }
}

pub fn unbind_technique_from_render_pass(techniques: &mut technique::TechniqueMap, pass: &Pass) {
    for (_, technique) in techniques.iter_mut() {
        technique::unbind_shader_program_from_technique(technique, &pass.program);
    }
}

pub fn resize_render_pass(pass: &mut Pass, width: u32, height: u32) {
    if width > 0 && height > 0 && (width != pass.width || height != pass.height) {
        let attachment_descriptors: Vec<PassAttachmentDescriptor> = pass
            .attachments
            .iter()
            .map(|attachment| {
                let mut desc = attachment.desc.clone();

                if desc.width == pass.width && desc.height == pass.height {
                    desc.width = width;
                    desc.height = height;
                } else {
                    log::log_warning(
                        "Failed to resize custom size frame buffer attachment!".to_string(),
                    );
                }

                return desc;
            })
            .collect();

        let attachments = create_pass_attachments(&attachment_descriptors);
        let fbo = create_framebuffer_object(&attachments);
        if let Result::Ok(fbo) = fbo {
            unsafe { gl::DeleteFramebuffers(1, &pass.fbo as *const u32) };
            delete_pass_attachments(&mut pass.attachments);

            pass.attachments = attachments;
            pass.fbo = fbo;
            pass.width = width;
            pass.height = height;
        }

        if let Err(msg) = fbo {
            log::log_error(format!("Failed to resize pass {0}", msg));
        }
    }
}

pub fn execute_render_pass(
    pass: &Pass,
    techniques: &HashMap<technique::Techniques, technique::Technique>,
    model: &model::DeviceModel,
) {
    let mut clear_mask: gl::types::GLbitfield = 0;

    for attachment in &pass.attachments {
        match attachment.desc.flavor {
            PassAttachmentType::Color(clear_color) => unsafe {
                gl::ClearColor(clear_color.x, clear_color.y, clear_color.z, clear_color.w);

                if attachment.desc.write {
                    gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
                } else {
                    gl::ColorMask(gl::FALSE, gl::FALSE, gl::FALSE, gl::FALSE);
                }

                if attachment.desc.clear {
                    clear_mask = clear_mask | gl::COLOR_BUFFER_BIT;
                }
            },
            PassAttachmentType::Depth(clear_depth, depth_func) => unsafe {
                gl::Enable(gl::DEPTH_TEST);
                gl::ClearDepth(clear_depth as f64);
                gl::DepthFunc(depth_func);

                if attachment.desc.write {
                    gl::DepthMask(gl::TRUE);
                } else {
                    gl::DepthMask(gl::FALSE);
                }

                if attachment.desc.clear {
                    clear_mask = clear_mask | gl::DEPTH_BUFFER_BIT;
                }
            },
        }
    }

    unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, pass.fbo);
        gl::Clear(clear_mask);
        gl::Viewport(0, 0, pass.width as i32, pass.height as i32);
        gl::UseProgram(pass.program.handle);
    }

    for mesh in &model.meshes {
        unsafe {
            gl::BindVertexArray(mesh.vao);
        }

        bind_material(
            &pass.program,
            &model.materials[mesh.material_index as usize],
        );
        bind_dependencies(&pass.program, &pass.dependencies);

        for technique in &pass.techniques {
            technique::update_per_frame_uniforms(
                &pass.program,
                &techniques.get(&technique).unwrap().per_frame_uniforms,
            );
            technique::update_per_model_uniform(
                &pass.program,
                &techniques.get(&technique).unwrap().per_model_uniforms,
                0,
            );
        }

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                (mesh.index_count * 3) as i32,
                gl::UNSIGNED_INT,
                null(),
            );
        }

        unbind_dependencies(&pass.program, &pass.dependencies);
        unbind_material(
            &pass.program,
            &model.materials[mesh.material_index as usize],
        );
    }

    unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, pass.fbo) };
}

pub fn blit_framebuffer_to_backbuffer(pass: &Pass, window: &app::Window) {
    unsafe {
        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, pass.fbo);
        gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
        gl::Viewport(0, 0, window.width as i32, window.height as i32);
        gl::Scissor(0, 0, window.width as i32, window.height as i32);
        gl::BlitFramebuffer(
            0,
            0,
            pass.width as i32,
            pass.height as i32,
            0,
            0,
            window.width as i32,
            window.height as i32,
            gl::COLOR_BUFFER_BIT,
            gl::LINEAR,
        );
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
}

fn bind_material(program: &shader::ShaderProgram, material: &model::DeviceMaterial) {
    if let Some(texture) = &material.albedo_texture {
        bind_texture(program.handle, texture);
    }
    if let Some(texture) = &material.normal_texture {
        bind_texture(program.handle, texture);
    }
    if let Some(texture) = &material.bump_texture {
        bind_texture(program.handle, texture);
    }
    if let Some(texture) = &material.metallic_texture {
        bind_texture(program.handle, texture);
    }
    if let Some(texture) = &material.roughness_texture {
        bind_texture(program.handle, texture);
    }
}

fn unbind_material(program: &shader::ShaderProgram, material: &model::DeviceMaterial) {
    if let Some(texture) = &material.albedo_texture {
        unbind_texture(program.handle, texture);
    }
    if let Some(texture) = &material.normal_texture {
        unbind_texture(program.handle, texture);
    }
    if let Some(texture) = &material.bump_texture {
        unbind_texture(program.handle, texture);
    }
    if let Some(texture) = &material.metallic_texture {
        unbind_texture(program.handle, texture);
    }
    if let Some(texture) = &material.roughness_texture {
        unbind_texture(program.handle, texture);
    }
}

fn bind_dependencies(program: &shader::ShaderProgram, dependencies: &Vec<model::Sampler2d>) {
    for dep in dependencies {
        bind_texture(program.handle, dep);
    }
}

fn unbind_dependencies(program: &shader::ShaderProgram, dependencies: &Vec<model::Sampler2d>) {
    for dep in dependencies {
        unbind_texture(program.handle, dep);
    }
}

fn bind_texture(program: u32, texture: &model::Sampler2d) {
    if let Some(binding) = texture.bindings.iter().find(|x| x.program == program) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 as u32 + binding.binding);
            gl::BindTexture(gl::TEXTURE_2D, texture.texture.handle);
        }
    }
}

fn unbind_texture(program: u32, texture: &model::Sampler2d) {
    if let Some(binding) = texture.bindings.iter().find(|x| x.program == program) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 as u32 + binding.binding);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
}

fn create_pass_attachments(descriptors: &Vec<PassAttachmentDescriptor>) -> Vec<PassAttachment> {
    let mut attachments: Vec<PassAttachment> = Vec::new();

    for desc in descriptors {
        let attachment = match &desc.source {
            PassAttachmentSource::Local => match &desc.flavor {
                PassAttachmentType::Color(_) => {
                    let device_texture_descriptor =
                        texture::create_color_attachment_device_texture_descriptor();
                    let attachment_host_texture = texture::create_empty_host_texture(
                        "color_attachment".to_string(),
                        desc.width as usize,
                        desc.height as usize,
                        texture::convert_gl_format_to_image_depth(device_texture_descriptor.format),
                    );
                    let attachment_device_texture = texture::create_device_texture(
                        attachment_host_texture.name.clone(),
                        &attachment_host_texture,
                        &device_texture_descriptor,
                    );
                    PassAttachment {
                        texture: attachment_device_texture,
                        desc: desc.clone(),
                    }
                }
                PassAttachmentType::Depth(_, _) => {
                    let device_texture_descriptor =
                        texture::create_depth_device_texture_descriptor();
                    let attachment_host_texture = texture::create_empty_host_texture(
                        "depth attachment".to_string(),
                        desc.width as usize,
                        desc.height as usize,
                        texture::convert_gl_format_to_image_depth(device_texture_descriptor.format),
                    );
                    let attachment_device_texture = texture::create_device_texture(
                        attachment_host_texture.name.clone(),
                        &attachment_host_texture,
                        &device_texture_descriptor,
                    );
                    PassAttachment {
                        texture: attachment_device_texture,
                        desc: desc.clone(),
                    }
                }
            },
            PassAttachmentSource::Remote(source) => PassAttachment {
                texture: source.device_texture.clone(),
                desc: desc.clone(),
            },
        };

        attachments.push(attachment);
    }

    attachments
}

fn delete_pass_attachments(attachments: &mut Vec<PassAttachment>) {
    for attachment in attachments.iter() {
        if let PassAttachmentSource::Local = attachment.desc.source {
            unsafe { gl::DeleteTextures(1, &attachment.texture.handle as *const u32) };
        }
    }
    attachments.clear();
}

// fn bind_pass_attachments_to_shader_program() -> Result<(), String>
// {

// }

fn create_framebuffer_object(attachments: &Vec<PassAttachment>) -> Result<u32, String> {
    let mut handle: u32 = 0;
    unsafe { gl::GenFramebuffers(1, &mut handle as *mut u32) };
    if handle == 0 {
        return Result::Err("Failed to create framebuffer object".to_string());
    }

    unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, handle) };
    let mut color_attachment_count: u32 = 0;
    let mut depth_attachment_count: u32 = 0;
    let mut draw_attachments: Vec<u32> = Vec::new();

    for attachment in attachments {
        unsafe { gl::ActiveTexture(gl::TEXTURE0) };
        unsafe { gl::BindTexture(gl::TEXTURE_2D, attachment.texture.handle) };

        let attachment_texture_handle = match &attachment.desc.source {
            PassAttachmentSource::Local => attachment.texture.handle,
            PassAttachmentSource::Remote(source) => source.device_texture.handle,
        };

        match attachment.desc.flavor {
            PassAttachmentType::Color(_) => {
                let attachment_index =
                    (gl::COLOR_ATTACHMENT0 as u32 + color_attachment_count) as gl::types::GLenum;

                unsafe {
                    gl::FramebufferTexture2D(
                        gl::FRAMEBUFFER,
                        attachment_index,
                        gl::TEXTURE_2D,
                        attachment_texture_handle,
                        0,
                    );
                }
                draw_attachments.push(attachment_index);

                color_attachment_count += 1;
            }
            PassAttachmentType::Depth(_, _) => {
                assert!(
                    depth_attachment_count == 0,
                    "There can only be 1 depth attachment"
                );
                unsafe {
                    gl::FramebufferTexture2D(
                        gl::FRAMEBUFFER,
                        gl::DEPTH_ATTACHMENT,
                        gl::TEXTURE_2D,
                        attachment_texture_handle,
                        0,
                    );
                }
                depth_attachment_count += 1;
            }
        }
    }

    unsafe { gl::DrawBuffers(draw_attachments.len() as i32, draw_attachments.as_ptr()) };
    if unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE } {
        return Result::Err("Framebuffer is not complete".to_string());
    }

    Result::Ok(handle)
}
