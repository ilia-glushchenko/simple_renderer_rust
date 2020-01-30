use crate::app;
use crate::math;
use crate::pass;
use crate::shader;
use crate::technique;

pub fn create_render_pipeline(
    techniques: &mut technique::TechniqueMap,
    window: &app::Window,
) -> Result<Vec<pass::Pass>, String> {
    let depth_pre_pass_descriptor = pass::PassDescriptor {
        name: "depth pre-pass".to_string(),
        program: shader::HostShaderProgramDescriptor {
            name: "depth pre-pass".to_string(),
            vert_shader_file_path: "shaders/depth_pre_pass.vert".to_string(),
            frag_shader_file_path: "shaders/depth_pre_pass.frag".to_string(),
        },
        techniques: vec![technique::Techniques::MVP],
        attachments: vec![pass::PassAttachmentDescriptor {
            flavor: pass::PassAttachmentType::Depth(1., gl::LESS),
            source: pass::PassAttachmentSource::Local,
            clear: true,
            write: true,
            width: window.width,
            height: window.height,
        }],
        dependencies: Vec::new(),
        width: window.width,
        height: window.height,
    };
    let depth_pre_pass = pass::create_render_pass(depth_pre_pass_descriptor);
    if let Err(msg) = depth_pre_pass {
        return Err(msg);
    }
    let depth_pre_pass = depth_pre_pass.unwrap();
    pass::bind_technique_to_render_pass(techniques, &depth_pre_pass);

    let lighting_pass_descriptor = pass::PassDescriptor {
        name: "lighting".to_string(),
        program: shader::HostShaderProgramDescriptor {
            name: "lighting".to_string(),
            vert_shader_file_path: "shaders/pass_through.vert".to_string(),
            frag_shader_file_path: "shaders/pass_through.frag".to_string(),
        },
        techniques: vec![technique::Techniques::MVP],
        attachments: vec![
            pass::PassAttachmentDescriptor {
                flavor: pass::PassAttachmentType::Depth(1., gl::EQUAL),
                source: pass::PassAttachmentSource::Remote(pass::RemotePassAttachmentSource {
                    pipeline_index: 0,
                    attachment_index: 0,
                    device_texture: depth_pre_pass.attachments[0].texture.clone(),
                }),
                clear: false,
                write: false,
                width: window.width,
                height: window.height,
            },
            pass::PassAttachmentDescriptor {
                flavor: pass::PassAttachmentType::Color(math::Vec4f {
                    x: 1.,
                    y: 1.,
                    z: 1.,
                    w: 1.,
                }),
                source: pass::PassAttachmentSource::Local,
                clear: true,
                write: true,
                width: window.width,
                height: window.height,
            },
        ],
        dependencies: Vec::new(),
        width: window.width,
        height: window.height,
    };
    let lighting_pass = pass::create_render_pass(lighting_pass_descriptor);
    if let Err(msg) = lighting_pass {
        return Err(msg);
    }
    let lighting_pass = lighting_pass.unwrap();
    pass::bind_technique_to_render_pass(techniques, &lighting_pass);

    Ok(vec![depth_pre_pass, lighting_pass])
}

pub fn delete_render_pipeline(
    techniques: &mut technique::TechniqueMap,
    pipeline: &mut Vec<pass::Pass>,
) {
    for pass in pipeline.iter_mut() {
        pass::unbind_technique_from_render_pass(techniques, &pass);
        pass::delete_render_pass(pass);
    }
}

pub fn resize_render_pipeline(window: &app::Window, pipeline: &mut Vec<pass::Pass>) {
    for i in 0..pipeline.len() {
        for j in 0..pipeline[i].attachments.len() {
            if let pass::PassAttachmentSource::Remote(source) =
                pipeline[i].attachments[j].desc.source.clone()
            {
                pipeline[i].attachments[j].desc.source =
                    pass::PassAttachmentSource::Remote(pass::RemotePassAttachmentSource {
                        pipeline_index: source.pipeline_index,
                        attachment_index: source.attachment_index,
                        device_texture: pipeline[source.pipeline_index].attachments
                            [source.attachment_index]
                            .texture
                            .clone(),
                    });
            }
        }

        pass::resize_render_pass(&mut pipeline[i], window.width, window.height);
    }
}
