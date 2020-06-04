extern crate gl;
use crate::asset::{material, model};
use crate::core::{app, tech};
use crate::gl::{shader, tex, uniform};
use crate::helpers::log;
use crate::math;
use std::collections::HashMap;
use std::ptr::null;
use std::rc::Rc;

pub struct Framebuffer {
    pub handle: u32,
    pub attachments: Vec<PassAttachment>,
}

impl Framebuffer {
    pub fn new(attachments: Vec<PassAttachment>) -> Result<Framebuffer, String> {
        let mut handle: u32 = 0;
        unsafe { gl::GenFramebuffers(1, &mut handle as *mut u32) };
        if handle == 0 {
            return Result::Err("Failed to create framebuffer object".to_string());
        }

        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, handle) };
        let mut color_attachment_count: u32 = 0;
        let mut depth_attachment_count: u32 = 0;
        let mut draw_attachments: Vec<u32> = Vec::new();

        for attachment in attachments.iter() {
            let attachment_texture = match &attachment.desc.source {
                PassTextureSource::ThisPass => attachment.texture.clone(),
                PassTextureSource::OtherPass(source) => source.device_texture.clone(),
                PassTextureSource::FreeTexture(texture) => texture.clone(),
            };

            unsafe { gl::ActiveTexture(gl::TEXTURE0) };
            unsafe { gl::BindTexture(attachment_texture.target, attachment_texture.handle) };

            match attachment.desc.flavor {
                PassAttachmentType::Color(_) => {
                    let attachment_index = (gl::COLOR_ATTACHMENT0 as u32 + color_attachment_count)
                        as gl::types::GLenum;

                    unsafe {
                        gl::FramebufferTexture2D(
                            gl::FRAMEBUFFER,
                            attachment_index,
                            attachment.desc.textarget,
                            attachment_texture.handle,
                            attachment.desc.mip_level,
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
                            attachment.desc.textarget,
                            attachment_texture.handle,
                            attachment.desc.mip_level,
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

        Result::Ok(Framebuffer {
            handle,
            attachments,
        })
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe { gl::DeleteFramebuffers(1, &self.handle as *const u32) };
    }
}

#[derive(Copy, Clone)]
pub enum PassAttachmentType {
    Color(math::Vec4f),
    Depth(f32, gl::types::GLenum),
}

#[derive(Clone)]
pub struct OtherPassTextureSource {
    pub pipeline_index: usize,
    pub attachment_index: usize,
    pub device_texture: Rc<tex::DeviceTexture>,
}

#[derive(Clone)]
pub enum PassTextureSource {
    ThisPass,
    OtherPass(OtherPassTextureSource),
    FreeTexture(Rc<tex::DeviceTexture>),
}

#[derive(Clone)]
pub struct PassAttachmentDescriptor {
    pub texture_desc: tex::Descriptor,
    pub flavor: PassAttachmentType,
    pub source: PassTextureSource,
    pub textarget: gl::types::GLenum,
    pub write: bool,
    pub clear: bool,
    pub width: u32,
    pub height: u32,
    pub mip_level: i32,
}

#[derive(Clone)]
pub struct PassAttachment {
    pub texture: Rc<tex::DeviceTexture>,
    pub desc: PassAttachmentDescriptor,
}

#[derive(Clone)]
pub struct PassDependencyDescriptor {
    pub name: String,
    pub source: OtherPassTextureSource,
}

#[derive(Clone)]
pub struct PassDependency {
    pub sampler: uniform::TextureSampler,
    pub desc: PassDependencyDescriptor,
}

#[derive(Clone)]
pub struct PassDescriptor {
    pub name: String,
    pub program: shader::HostShaderProgramDescriptor,
    pub techniques: Vec<tech::Techniques>,

    pub attachments: Vec<PassAttachmentDescriptor>,
    pub dependencies: Vec<PassDependencyDescriptor>,

    pub width: u32,
    pub height: u32,
}

pub struct Pass {
    pub name: String,
    pub program: shader::ShaderProgram,
    pub program_desc: shader::HostShaderProgramDescriptor,
    pub techniques: Vec<tech::Techniques>,
    pub dependencies: Vec<PassDependency>,

    pub fbo: Framebuffer,
    pub width: u32,
    pub height: u32,
}

impl Pass {
    pub fn new(desc: PassDescriptor) -> Result<Pass, String> {
        let device_program = shader::ShaderProgram::new(&desc.program);
        if let Err(msg) = device_program {
            return Err(msg);
        }
        let device_program = device_program.unwrap();

        let dependencies = create_pass_dependencies(&device_program, &desc.dependencies);

        let attachments = create_pass_attachments(&desc.attachments);
        let framebuffer_object = Framebuffer::new(attachments);
        if let Result::Err(msg) = framebuffer_object {
            return Result::Err(msg);
        }

        Result::Ok(Pass {
            name: desc.name.clone(),
            program: device_program,
            program_desc: desc.program.clone(),
            techniques: desc.techniques.clone(),
            dependencies,
            fbo: framebuffer_object.unwrap(),
            width: desc.width,
            height: desc.height,
        })
    }

    pub fn recreate_attachments(
        &mut self,
        descriptors: &Vec<PassAttachmentDescriptor>,
    ) -> Result<(), String> {
        let attachments = create_pass_attachments(descriptors);
        let framebuffer_object = Framebuffer::new(attachments);
        if let Result::Err(msg) = framebuffer_object {
            return Result::Err(msg);
        }
        self.fbo = framebuffer_object.unwrap();

        Ok(())
    }

    pub fn execute(&self, techniques: &tech::TechniqueContainer, model: &model::DeviceModel) {
        let mut clear_mask: gl::types::GLbitfield = 0;

        for attachment in &self.fbo.attachments {
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
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo.handle);
            gl::Clear(clear_mask);
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
            gl::UseProgram(self.program.handle);
        }

        for technique_name in &self.techniques {
            let technique = &techniques.map.get(&technique_name).unwrap();

            uniform::update_per_frame_uniforms(&self.program, &technique.per_frame_uniforms);
            for texture in &technique.textures {
                bind_texture(self.program.handle, texture);
            }
        }

        for (i, mesh) in model.meshes.iter().enumerate() {
            unsafe {
                gl::BindVertexArray(mesh.vao);
            }

            bind_material(
                &self.program,
                &model.materials[mesh.material_index as usize],
            );
            bind_dependencies(&self.program, &self.dependencies);

            for technique in &self.techniques {
                uniform::update_per_model_uniform(
                    &self.program,
                    &techniques.map.get(&technique).unwrap().per_model_uniforms,
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

            unbind_dependencies(&self.program, &self.dependencies);
            unbind_material(
                &self.program,
                &model.materials[mesh.material_index as usize],
            );
        }

        for technique_name in &self.techniques {
            let technique = &techniques.map.get(&technique_name).unwrap();
            for texture in &technique.textures {
                unbind_texture(self.program.handle, texture);
            }
        }

        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo.handle) };
    }
}

pub fn resize_render_pass(pass: &mut Pass, width: u32, height: u32) {
    if width > 0 && height > 0 && (width != pass.width || height != pass.height) {
        let attachment_descriptors: Vec<PassAttachmentDescriptor> = pass
            .fbo
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

        let dependency_descriptors: Vec<PassDependencyDescriptor> = pass
            .dependencies
            .iter()
            .map(|dependency| {
                return PassDependencyDescriptor {
                    name: dependency.desc.name.clone(),
                    source: dependency.desc.source.clone(),
                };
            })
            .collect();

        let dependencies = create_pass_dependencies(&pass.program, &dependency_descriptors);

        let attachments = create_pass_attachments(&attachment_descriptors);
        match Framebuffer::new(attachments) {
            Result::Ok(fbo) => {
                unsafe { gl::DeleteFramebuffers(1, &pass.fbo.handle as *const u32) };
                pass.dependencies = dependencies;
                pass.fbo = fbo;
                pass.width = width;
                pass.height = height;
            }
            Result::Err(msg) => {
                log::log_error(format!("Failed to resize pass {0}", msg));
                panic!(msg);
            }
        }
    }
}

pub fn is_render_pass_valid(
    pass: &Pass,
    techniques: &tech::TechniqueContainer,
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

        // All uniforms must be provided by techniques or materials
        // A uniform cannot be provided by more than one technique
        for technique in &pass.techniques {
            for uniform in techniques.map[technique].per_frame_uniforms.vec1f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques.map[technique].per_frame_uniforms.vec1u.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques.map[technique].per_frame_uniforms.vec2f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques.map[technique].per_frame_uniforms.vec3f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques.map[technique].per_frame_uniforms.mat4x4f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }

            for uniform in techniques.map[technique].per_model_uniforms.vec1f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques.map[technique].per_model_uniforms.vec1u.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques.map[technique].per_model_uniforms.vec2f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques.map[technique].per_model_uniforms.vec3f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
            for uniform in techniques.map[technique].per_model_uniforms.mat4x4f.iter() {
                *scalar_uniforms.entry(&uniform.name).or_insert(0) += 1;
            }
        }
        for (uniform, count) in &scalar_uniforms {
            if *count > 1 {
                log::log_error(format!(
                    "Render pass '{}' is invalid! Uniform '{}' provided '{}' times by techniques.",
                    pass.name, uniform, count
                ));
            }
        }

        //A uniform cannot be provided by both material and technique
        for material in &device_model.materials {
            for property_1u in &material.properties_1u {
                *scalar_uniforms.entry(&property_1u.value.name).or_insert(0) += 1;
            }
            for property_1f in &material.properties_1f {
                *scalar_uniforms.entry(&property_1f.value.name).or_insert(0) += 1;
            }
            for property_3f in &material.properties_3f {
                *scalar_uniforms.entry(&property_3f.value.name).or_insert(0) += 1;
            }
        }
        for (uniform, count) in &scalar_uniforms {
            if *count > 1 && *count != device_model.materials.len() as u32 {
                log::log_error(format!(
                    "Render pass '{}' is invalid!
                     Uniform '{}'  provided '{}' times by techniques and materials.",
                    pass.name, uniform, count
                ));
            } else if *count == 0 {
                log::log_error(format!(
                    "Render pass '{}' is invalid! Uniform '{}' not provided by tehcniques or materials.",
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
            let textures: Vec<&uniform::TextureSampler> = device_model.materials
                [mesh.material_index]
                .properties_samplers
                .iter()
                .map(|x| &x.value)
                .collect();

            for sampler in &pass.program.samplers {
                let dependency = pass
                    .dependencies
                    .iter()
                    .find(|d| d.sampler.name == sampler.name);

                let material = textures.iter().find(|t| t.name == sampler.name);

                let mut technique_textures: Vec<&uniform::TextureSampler> = Vec::new();
                for technique in &pass.techniques {
                    if let Some(texture) = &techniques.map[technique]
                        .textures
                        .iter()
                        .find(|t| t.name == sampler.name)
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

pub fn blit_framebuffer_to_backbuffer(pass: &Pass, app: &app::App) {
    unsafe {
        gl::BindFramebuffer(gl::READ_FRAMEBUFFER, pass.fbo.handle);
        gl::ReadBuffer(gl::COLOR_ATTACHMENT0);
        gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
        gl::Viewport(0, 0, app.width as i32, app.height as i32);
        gl::Scissor(0, 0, app.width as i32, app.height as i32);
        gl::BlitFramebuffer(
            0,
            0,
            pass.width as i32,
            pass.height as i32,
            0,
            0,
            app.width as i32,
            app.height as i32,
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
            PassTextureSource::ThisPass => match &desc.flavor {
                PassAttachmentType::Color(_) => {
                    let attachment_host_texture = tex::HostTexture::empty(
                        "color_attachment".to_string(),
                        desc.width as usize,
                        desc.height as usize,
                        tex::convert_gl_format_to_image_depth(desc.texture_desc.format),
                    );
                    let attachment_device_texture =
                        tex::DeviceTexture::new(&attachment_host_texture, &desc.texture_desc);
                    PassAttachment {
                        texture: attachment_device_texture,
                        desc: desc.clone(),
                    }
                }
                PassAttachmentType::Depth(_, _) => {
                    let device_texture_descriptor =
                        tex::Descriptor::new(tex::DescriptorType::Depth);
                    let attachment_host_texture = tex::HostTexture::empty(
                        "depth attachment".to_string(),
                        desc.width as usize,
                        desc.height as usize,
                        tex::convert_gl_format_to_image_depth(device_texture_descriptor.format),
                    );
                    let attachment_device_texture = tex::DeviceTexture::new(
                        &attachment_host_texture,
                        &device_texture_descriptor,
                    );
                    PassAttachment {
                        texture: attachment_device_texture,
                        desc: desc.clone(),
                    }
                }
            },
            PassTextureSource::OtherPass(source) => PassAttachment {
                texture: source.device_texture.clone(),
                desc: desc.clone(),
            },
            PassTextureSource::FreeTexture(texture) => PassAttachment {
                texture: texture.clone(),
                desc: desc.clone(),
            },
        };

        attachments.push(attachment);
    }

    attachments
}

pub fn create_pass_dependencies(
    program: &shader::ShaderProgram,
    descriptors: &Vec<PassDependencyDescriptor>,
) -> Vec<PassDependency> {
    let mut dependencies: Vec<PassDependency> = Vec::new();

    for desc in descriptors {
        dependencies.push(PassDependency {
            sampler: uniform::TextureSampler::new(&desc.name, desc.source.device_texture.clone()),
            desc: desc.clone(),
        })
    }

    for dependency in &mut dependencies {
        dependency.sampler.bind_shader_program(&program);
    }

    dependencies
}

fn bind_material(program: &shader::ShaderProgram, material: &material::DeviceMaterial) {
    for property_sampler in &material.properties_samplers {
        bind_texture(program.handle, &property_sampler.value);
    }

    for property_1u in &material.properties_1u {
        uniform::update_per_frame_uniforms_vec1u(program, std::slice::from_ref(&property_1u.value));
    }

    for property_1f in &material.properties_1f {
        uniform::update_per_frame_uniforms_vec1f(program, std::slice::from_ref(&property_1f.value));
    }

    for property_3f in &material.properties_3f {
        uniform::update_per_frame_uniforms_vec3f(program, std::slice::from_ref(&property_3f.value));
    }
}

fn unbind_material(program: &shader::ShaderProgram, material: &material::DeviceMaterial) {
    for property_sampler in &material.properties_samplers {
        unbind_texture(program.handle, &property_sampler.value);
    }
}

fn bind_dependencies(program: &shader::ShaderProgram, dependencies: &Vec<PassDependency>) {
    for dep in dependencies {
        bind_texture(program.handle, &dep.sampler);
    }
}

fn unbind_dependencies(program: &shader::ShaderProgram, dependencies: &Vec<PassDependency>) {
    for dep in dependencies {
        unbind_texture(program.handle, &dep.sampler);
    }
}

fn bind_texture(program: u32, sampler: &uniform::TextureSampler) {
    if let Some(binding) = sampler.bindings.iter().find(|x| x.program == program) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 as u32 + binding.binding);
            gl::BindTexture(sampler.texture.target, sampler.texture.handle);
        }
    }
}

fn unbind_texture(program: u32, sampler: &uniform::TextureSampler) {
    if let Some(binding) = sampler.bindings.iter().find(|x| x.program == program) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 as u32 + binding.binding);
            gl::BindTexture(sampler.texture.target, 0);
        }
    }
}
