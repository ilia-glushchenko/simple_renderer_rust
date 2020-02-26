extern crate gl;
use crate::app;
use crate::log;
use crate::math;
use crate::model;
use crate::shader;
use crate::tech;
use crate::tex;
use std::collections::HashMap;
use std::ptr::null;

#[derive(Copy, Clone)]
pub enum PassAttachmentType {
    Color(math::Vec4f),
    Depth(f32, gl::types::GLenum),
}

#[derive(Clone)]
pub struct OtherPassAttachmentSource {
    pub pipeline_index: usize,
    pub attachment_index: usize,
    pub device_texture: tex::DeviceTexture,
}

#[derive(Clone)]
pub enum PassAttachmentSource {
    ThisPass,
    OtherPass(OtherPassAttachmentSource),
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
    pub texture: tex::DeviceTexture,
    pub desc: PassAttachmentDescriptor,
}

#[derive(Clone)]
pub struct PassDescriptor {
    pub name: String,
    pub program: shader::HostShaderProgramDescriptor,
    pub techniques: Vec<tech::Techniques>,

    pub attachments: Vec<PassAttachmentDescriptor>,
    pub dependencies: Vec<model::TextureSampler>,

    pub width: u32,
    pub height: u32,
}

#[derive(Clone)]
pub struct Pass {
    pub name: String,
    pub program: shader::ShaderProgram,
    pub techniques: Vec<tech::Techniques>,
    pub desc: PassDescriptor,

    pub attachments: Vec<PassAttachment>,
    pub dependencies: Vec<model::TextureSampler>,

    pub fbo: u32,
    pub width: u32,
    pub height: u32,
}

pub fn create_render_pass(mut desc: PassDescriptor) -> Result<Pass, String> {
    let device_program = shader::create_shader_program(&desc.program);
    if let Err(msg) = device_program {
        return Err(msg);
    }
    let device_program = device_program.unwrap();

    for dependency in &mut desc.dependencies {
        tech::bind_shader_program_to_texture(&device_program, dependency);
    }

    let attachments = create_pass_attachments(&desc.attachments);
    let framebuffer_object = create_framebuffer_object(&attachments);
    if let Result::Err(msg) = framebuffer_object {
        return Result::Err(msg);
    }

    Result::Ok(Pass {
        name: desc.name.clone(),
        program: device_program,
        techniques: desc.techniques.clone(),
        desc: desc.clone(),
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

pub fn create_framebuffer_object(attachments: &Vec<PassAttachment>) -> Result<u32, String> {
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
            PassAttachmentSource::ThisPass => attachment.texture.handle,
            PassAttachmentSource::OtherPass(source) => source.device_texture.handle,
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

pub fn bind_device_model_to_render_pass(device_model: &mut model::DeviceModel, pass: &Pass) {
    for device_mesh in &mut device_model.meshes {
        tech::bind_device_mesh_to_shader_program(device_mesh, &pass.program);
    }
    for device_material in &mut device_model.materials {
        tech::bind_shader_program_to_material(device_material, &pass.program);
    }
}

pub fn unbind_device_model_from_render_pass(
    device_model: &mut model::DeviceModel,
    pass_program_handle: u32,
) {
    for device_material in &mut device_model.materials {
        tech::unbind_shader_program_from_material(device_material, pass_program_handle);
    }
}

pub fn bind_technique_to_render_pass(techniques: &mut tech::TechniqueMap, pass: &Pass) {
    for (_, technique) in techniques.iter_mut() {
        tech::bind_shader_program_to_technique(technique, &pass.program);
    }
}

pub fn unbind_technique_from_render_pass(
    techniques: &mut tech::TechniqueMap,
    pass_program_handle: u32,
) {
    for (_, technique) in techniques.iter_mut() {
        tech::unbind_shader_program_from_technique(technique, pass_program_handle);
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

pub fn is_render_pass_valid(
    pass: &Pass,
    techniques: &tech::TechniqueMap,
    device_model: &model::DeviceModel,
) -> Result<(), String> {
    // Check scalar uniforms
    {
        let mut scalar_uniforms: HashMap<&String, u32> = pass
            .program
            .scalar_uniforms
            .iter()
            .map(|x| (&x.name, 0))
            .collect();

        for technique in &pass.techniques {
            for uniform in techniques[technique].per_frame_uniforms.vec1f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques[technique].per_frame_uniforms.vec1u.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques[technique].per_frame_uniforms.vec2f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques[technique].per_frame_uniforms.vec3f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques[technique].per_frame_uniforms.mat4x4f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }

            for uniform in techniques[technique].per_model_uniforms.vec1f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques[technique].per_model_uniforms.vec1u.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques[technique].per_model_uniforms.vec2f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques[technique].per_model_uniforms.vec3f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques[technique].per_model_uniforms.mat4x4f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
        }

        for (uniform, count) in scalar_uniforms {
            if count > 1 {
                log::log_error(format!(
                    "Render pass '{}' is invalid! Uniform '{}' provided '{}' times.",
                    pass.name, uniform, count
                ));
            } else if count == 0 {
                log::log_error(format!(
                    "Render pass '{}' is invalid! Uniform '{}' not provided by tehcniques.",
                    pass.name, uniform
                ));
            }
        }
    }

    // Check attributes
    {
        for (i, mesh) in device_model.meshes.iter().enumerate() {
            for shader_attribute in &pass.program.attributes {
                if let None = mesh
                    .attributes
                    .iter()
                    .find(|a| a.name == shader_attribute.name)
                {
                    return Err(format!(
                        "Render pass '{}' is invalid! \
                         Mesh #'{}' does not provide shader attribute '{}'.",
                        pass.name, i, shader_attribute.name
                    ));
                }
            }
        }
    }

    // Check texture samplers
    {
        for (i, mesh) in device_model.meshes.iter().enumerate() {
            let mut textures: Vec<&model::TextureSampler> = Vec::new();

            if let Some(texture) = &device_model.materials[mesh.material_index].albedo_texture {
                textures.push(&texture);
            }
            if let Some(texture) = &device_model.materials[mesh.material_index].normal_texture {
                textures.push(&texture);
            }
            if let Some(texture) = &device_model.materials[mesh.material_index].bump_texture {
                textures.push(&texture);
            }
            if let Some(texture) = &device_model.materials[mesh.material_index].metallic_texture {
                textures.push(&texture);
            }
            if let Some(texture) = &device_model.materials[mesh.material_index].roughness_texture {
                textures.push(&texture);
            }

            for sampler in &pass.program.samplers {
                let dependency = pass
                    .dependencies
                    .iter()
                    .find(|d| d.texture.name == sampler.name);

                let material = textures.iter().find(|t| t.texture.name == sampler.name);

                let mut technique_textures: Vec<&model::TextureSampler> = Vec::new();
                for technique in &pass.techniques {
                    if let Some(texture) = &techniques[technique]
                        .textures
                        .iter()
                        .find(|t| t.texture.name == sampler.name)
                    {
                        technique_textures.push(texture);
                    }
                }
                if technique_textures.len() > 1 {
                    return Err(format!(
                        "Render pass '{}' is invalid! \
                         Shader's '{}' Texture Sampler '{}' bound by '{}' techniques. Can only handle 1.",
                         pass.name, pass.program.name, sampler.name,  technique_textures.len()
                    ));
                }

                //ToDo: Make better handling of those errors
                if dependency.is_none() && material.is_none() && technique_textures.is_empty() {
                    log::log_error(format!(
                        "Render pass '{}' is invalid! \
                         Shader's '{}' Texture Sampler '{}' is not bound by dependencies, techniques or material for mesh #'{}'.",
                         pass.name, pass.program.name, sampler.name, i
                    ));
                } else if dependency.is_some()
                    && material.is_some()
                    && technique_textures.is_empty()
                {
                    log::log_error(format!(
                        "Render pass '{}' is invalid! \
                         Shader's '{}' Texture Sampler '{}' is bound both by pass dependency and material for mesh #'{}'.",
                         pass.name, pass.program.name, sampler.name, i
                    ));
                } else if dependency.is_some()
                    && material.is_none()
                    && !technique_textures.is_empty()
                {
                    log::log_error(format!(
                        "Render pass '{}' is invalid! \
                         Shader's '{}' Texture Sampler '{}' is bound both by pass dependency and a technique.",
                         pass.name, pass.program.name, sampler.name
                    ));
                } else if dependency.is_none()
                    && material.is_some()
                    && !technique_textures.is_empty()
                {
                    log::log_error(format!(
                        "Render pass '{}' is invalid! \
                         Shader's '{}' Texture Sampler '{}' is bound both by technique and material for mesh #'{}'.",
                         pass.name, pass.program.name, sampler.name, i
                    ));
                }
            }
        }
    }

    Ok(())
}

pub fn execute_render_pass(
    pass: &Pass,
    techniques: &HashMap<tech::Techniques, tech::Technique>,
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

    for technique_name in &pass.techniques {
        let technique = &techniques.get(&technique_name).unwrap();

        tech::update_per_frame_uniforms(&pass.program, &technique.per_frame_uniforms);
        for texture in &technique.textures {
            bind_texture(pass.program.handle, texture);
        }
    }

    for (i, mesh) in model.meshes.iter().enumerate() {
        unsafe {
            gl::BindVertexArray(mesh.vao);
        }

        bind_material(
            &pass.program,
            &model.materials[mesh.material_index as usize],
        );
        bind_dependencies(&pass.program, &pass.dependencies);

        for technique in &pass.techniques {
            tech::update_per_model_uniform(
                &pass.program,
                &techniques.get(&technique).unwrap().per_model_uniforms,
                i,
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

    for technique_name in &pass.techniques {
        let technique = &techniques.get(&technique_name).unwrap();
        for texture in &technique.textures {
            unbind_texture(pass.program.handle, texture);
        }
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

pub fn copy_framebuffer_to_texture(fbo: u32, width: u32, height: u32, target: gl::types::GLenum) {
    unsafe {
        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, fbo);
        gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        gl::CopyTexSubImage2D(target, 0, 0, 0, 0, 0, width as i32, height as i32);
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
}

pub fn create_pass_attachments(descriptors: &Vec<PassAttachmentDescriptor>) -> Vec<PassAttachment> {
    let mut attachments: Vec<PassAttachment> = Vec::new();

    for desc in descriptors {
        let attachment = match &desc.source {
            PassAttachmentSource::ThisPass => match &desc.flavor {
                PassAttachmentType::Color(_) => {
                    let device_texture_descriptor =
                        tex::create_color_attachment_device_texture_descriptor();
                    let attachment_host_texture = tex::create_empty_host_texture(
                        "color_attachment".to_string(),
                        desc.width as usize,
                        desc.height as usize,
                        tex::convert_gl_format_to_image_depth(device_texture_descriptor.format),
                    );
                    let attachment_device_texture = tex::create_device_texture(
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
                    let device_texture_descriptor = tex::create_depth_device_texture_descriptor();
                    let attachment_host_texture = tex::create_empty_host_texture(
                        "depth attachment".to_string(),
                        desc.width as usize,
                        desc.height as usize,
                        tex::convert_gl_format_to_image_depth(device_texture_descriptor.format),
                    );
                    let attachment_device_texture = tex::create_device_texture(
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
            PassAttachmentSource::OtherPass(source) => PassAttachment {
                texture: source.device_texture.clone(),
                desc: desc.clone(),
            },
        };

        attachments.push(attachment);
    }

    attachments
}

pub fn delete_pass_attachments(attachments: &mut Vec<PassAttachment>) {
    for attachment in attachments.iter() {
        if let PassAttachmentSource::ThisPass = attachment.desc.source {
            unsafe { gl::DeleteTextures(1, &attachment.texture.handle as *const u32) };
        }
    }
    attachments.clear();
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

    tech::update_per_frame_uniforms_vec3f(program, std::slice::from_ref(&material.scalar_albedo));
    tech::update_per_frame_uniforms_vec1f(
        program,
        std::slice::from_ref(&material.scalar_roughness),
    );
    tech::update_per_frame_uniforms_vec1f(
        program,
        std::slice::from_ref(&material.scalar_metalness),
    );
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

fn bind_dependencies(program: &shader::ShaderProgram, dependencies: &Vec<model::TextureSampler>) {
    for dep in dependencies {
        bind_texture(program.handle, dep);
    }
}

fn unbind_dependencies(program: &shader::ShaderProgram, dependencies: &Vec<model::TextureSampler>) {
    for dep in dependencies {
        unbind_texture(program.handle, dep);
    }
}

fn bind_texture(program: u32, sampler: &model::TextureSampler) {
    if let Some(binding) = sampler.bindings.iter().find(|x| x.program == program) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 as u32 + binding.binding);
            gl::BindTexture(sampler.texture.target, sampler.texture.handle);
        }
    }
}

fn unbind_texture(program: u32, sampler: &model::TextureSampler) {
    if let Some(binding) = sampler.bindings.iter().find(|x| x.program == program) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 as u32 + binding.binding);
            gl::BindTexture(sampler.texture.target, 0);
        }
    }
}
