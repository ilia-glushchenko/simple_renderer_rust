use crate::loader;
use crate::math;
use crate::model;
use crate::pass;
use crate::shader;
use crate::tech;
use crate::techniques;
use crate::tex;
use std::f32;
use std::ffi::c_void;
use std::path::Path;
use std::ptr::null;

pub fn create_specular_cube_map_texture(host_texture: &tex::HostTexture) -> tex::DeviceTexture {
    let width: u32 = 2048;
    let height: u32 = 2048;

    let (mut techs, mut pass, box_model) = create_hdri_2_cube_map_pass(host_texture, width, height);

    let fbos = draw_cubemap(&mut techs, &mut pass, &box_model);

    let desc = tex::create_spherical_hdri_texture_descriptor(&host_texture);
    let cubemap = create_cubemap(&desc, width, height, &fbos);

    cleanup_cubempa_fbos(fbos, &mut techs, pass, box_model);
    tech::delete_techniques(techs);

    cubemap
}

pub fn create_diffuse_cube_map_texture(cube_map: &tex::DeviceTexture) -> tex::DeviceTexture  {
    let width: u32 = 2048;
    let height: u32 = 2048;

    let (mut techs, mut pass, box_model) = create_cubemap_convolution_pass(cube_map, width, height);

    let fbos = draw_cubemap(&mut techs, &mut pass, &box_model);

    let desc = tex::create_color_attachment_device_texture_descriptor();
    let cubemap = create_cubemap(&desc, width, height, &fbos);

    cleanup_cubempa_fbos(fbos, &mut techs, pass, box_model);

    cubemap
}

struct PassFbo {
    fbo: u32,
    attachments: Vec<pass::PassAttachment>,
}

struct CubeMapFbos {
    nz: PassFbo,
    pz: PassFbo,
    py: PassFbo,
    ny: PassFbo,
    nx: PassFbo,
    px: PassFbo,
}

fn create_cubemap(
    desc: &tex::DeviceTextureDescriptor,
    width: u32,
    height: u32,
    fbos: &CubeMapFbos,
) -> tex::DeviceTexture {
    let mut handle: u32 = 0;

    unsafe {
        gl::GenTextures(1, &mut handle as *mut u32);
        assert!(handle != 0, "Failed to generate texture");
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, handle);

        for i in 0..6 {
            gl::TexImage2D(
                (gl::TEXTURE_CUBE_MAP_POSITIVE_X as usize + i) as gl::types::GLenum,
                0,
                desc.internal_format as i32,
                width as i32,
                height as i32,
                0,
                desc.format,
                desc.data_type,
                null() as *const c_void,
            );
        }

        pass::copy_framebuffer_to_texture(
            fbos.nz.fbo,
            width,
            height,
            gl::TEXTURE_CUBE_MAP_NEGATIVE_Z,
        );
        pass::copy_framebuffer_to_texture(
            fbos.pz.fbo,
            width,
            height,
            gl::TEXTURE_CUBE_MAP_POSITIVE_Z,
        );
        pass::copy_framebuffer_to_texture(
            fbos.nx.fbo,
            width,
            height,
            gl::TEXTURE_CUBE_MAP_NEGATIVE_X,
        );
        pass::copy_framebuffer_to_texture(
            fbos.px.fbo,
            width,
            height,
            gl::TEXTURE_CUBE_MAP_POSITIVE_X,
        );
        pass::copy_framebuffer_to_texture(
            fbos.ny.fbo,
            width,
            height,
            gl::TEXTURE_CUBE_MAP_NEGATIVE_Y,
        );
        pass::copy_framebuffer_to_texture(
            fbos.py.fbo,
            width,
            height,
            gl::TEXTURE_CUBE_MAP_POSITIVE_Y,
        );

        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, desc.s_wrap as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, desc.t_wrap as i32);
        gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, desc.r_wrap as i32);
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MAG_FILTER,
            desc.mag_filter as i32,
        );
        gl::TexParameteri(
            gl::TEXTURE_CUBE_MAP,
            gl::TEXTURE_MIN_FILTER,
            desc.min_filter as i32,
        );

        gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);
    }

    tex::DeviceTexture {
        name: "uSkyboxSamplerCube".to_string(),
        handle,
        target: gl::TEXTURE_CUBE_MAP,
    }
}

fn cleanup_cubempa_fbos(
    mut fbos: CubeMapFbos,
    techs: &mut tech::TechniqueMap,
    mut pass: pass::Pass,
    mut box_model: model::DeviceModel,
) {
    pass::delete_pass_attachments(&mut fbos.nz.attachments);
    unsafe { gl::DeleteFramebuffers(1, &fbos.nz.fbo as *const u32) };
    pass::delete_pass_attachments(&mut fbos.pz.attachments);
    unsafe { gl::DeleteFramebuffers(1, &fbos.pz.fbo as *const u32) };
    pass::delete_pass_attachments(&mut fbos.py.attachments);
    unsafe { gl::DeleteFramebuffers(1, &fbos.py.fbo as *const u32) };
    pass::delete_pass_attachments(&mut fbos.ny.attachments);
    unsafe { gl::DeleteFramebuffers(1, &fbos.ny.fbo as *const u32) };
    pass::delete_pass_attachments(&mut fbos.nx.attachments);
    unsafe { gl::DeleteFramebuffers(1, &fbos.nx.fbo as *const u32) };
    pass::delete_pass_attachments(&mut fbos.px.attachments);
    unsafe { gl::DeleteFramebuffers(1, &fbos.px.fbo as *const u32) };

    pass::unbind_technique_from_render_pass(techs, pass.program.handle);
    pass::unbind_device_model_from_render_pass(&mut box_model, pass.program.handle);
    pass::unbind_technique_from_render_pass(techs, pass.program.handle);
    pass::delete_render_pass(&mut pass);
    model::delete_device_model(box_model);
}

fn draw_cubemap(
    techs: &mut tech::TechniqueMap,
    pass: &mut pass::Pass,
    box_model: &model::DeviceModel,
) -> CubeMapFbos {
    CubeMapFbos {
        nz: draw_cubemap_face(
            techs,
            math::y_rotation_mat4x4(-f32::consts::PI / 2.),
            pass,
            &box_model,
        ),
        pz: draw_cubemap_face(
            techs,
            math::y_rotation_mat4x4(f32::consts::PI / 2.),
            pass,
            &box_model,
        ),
        py: draw_cubemap_face(
            techs,
            math::x_rotation_mat4x4(f32::consts::PI / 2.)
                * math::y_rotation_mat4x4(f32::consts::PI / 2.),
            pass,
            &box_model,
        ),
        ny: draw_cubemap_face(
            techs,
            math::x_rotation_mat4x4(-f32::consts::PI / 2.)
                * math::y_rotation_mat4x4(f32::consts::PI / 2.),
            pass,
            &box_model,
        ),
        nx: draw_cubemap_face(techs, math::identity_mat4x4(), pass, &box_model),
        px: draw_cubemap_face(
            techs,
            math::y_rotation_mat4x4(-f32::consts::PI),
            pass,
            &box_model,
        ),
    }
}

fn draw_cubemap_face(
    techs: &mut tech::TechniqueMap,
    view: math::Mat4x4f,
    pass: &mut pass::Pass,
    box_model: &model::DeviceModel,
) -> PassFbo {
    techniques::ibl::update(techs.get_mut(&tech::Techniques::IBL).unwrap(), view);
    pass::execute_render_pass(&pass, &techs, &box_model);
    let attachments = pass.attachments.clone();
    let fbo = pass.fbo;
    pass.attachments = create_attachments(pass.width, pass.height);
    pass.fbo = pass::create_framebuffer_object(&pass.attachments).unwrap();

    PassFbo { fbo, attachments }
}

fn create_hdri_2_cube_map_pass(
    host_texture: &tex::HostTexture,
    width: u32,
    height: u32,
) -> (tech::TechniqueMap, pass::Pass, model::DeviceModel) {
    let mut techs = tech::TechniqueMap::new();
    techs.insert(
        tech::Techniques::IBL,
        techniques::ibl::hdri2cube::create(
            math::perspective_projection_mat4x4(f32::consts::PI / 2.0, 1., 0.98, 1.01),
            tex::create_device_texture(
                "uHdriSampler2D".to_string(),
                &host_texture,
                &tex::create_spherical_hdri_texture_descriptor(&host_texture),
            ),
        ),
    );

    let pass_desc = pass::PassDescriptor {
        name: "HDRI 2 Cube".to_string(),
        program: shader::HostShaderProgramDescriptor {
            name: "HDIR 2 Cube".to_string(),
            vert_shader_file_path: "shaders/hdri2cube.vert".to_string(),
            frag_shader_file_path: "shaders/hdri2cube.frag".to_string(),
        },
        techniques: vec![tech::Techniques::IBL],

        attachments: vec![pass::PassAttachmentDescriptor {
            flavor: pass::PassAttachmentType::Color(math::zero_vec4()),
            source: pass::PassTextureSource::ThisPass,
            write: true,
            clear: true,
            width,
            height,
        }],
        dependencies: Vec::new(),

        width,
        height,
    };

    let pass = pass::create_render_pass(pass_desc).expect("Failed to create HDRI render pass.");

    let mut box_model = loader::load_device_model_from_obj(Path::new("data/models/box/box.obj"));
    pass::bind_technique_to_render_pass(&mut techs, &pass);
    pass::bind_device_model_to_render_pass(&mut box_model, &pass);
    if let Err(msg) = pass::is_render_pass_valid(&pass, &techs, &box_model) {
        panic!(msg);
    }

    (techs, pass, box_model)
}

fn create_cubemap_convolution_pass(
    cube_map: &tex::DeviceTexture,
    width: u32,
    height: u32,
) -> (tech::TechniqueMap, pass::Pass, model::DeviceModel)
{
    let mut techs = tech::TechniqueMap::new();
    techs.insert(
        tech::Techniques::IBL,
        techniques::ibl::cubemap_convolution::create(
            math::perspective_projection_mat4x4(f32::consts::PI / 2.0, 1., 0.98, 1.01),
            tex::DeviceTexture{
                name: "uSkyboxSamplerCube".to_string(),
                handle: cube_map.handle,
                target: cube_map.target,
            }
        ),
    );

    let pass_desc = pass::PassDescriptor {
        name: "Cubemap Convolution".to_string(),
        program: shader::HostShaderProgramDescriptor {
            name: "Cubemap Convolution".to_string(),
            vert_shader_file_path: "shaders/cube_map_convolution.vert".to_string(),
            frag_shader_file_path: "shaders/cube_map_convolution.frag".to_string(),
        },
        techniques: vec![tech::Techniques::IBL],

        attachments: vec![pass::PassAttachmentDescriptor {
            flavor: pass::PassAttachmentType::Color(math::zero_vec4()),
            source: pass::PassTextureSource::ThisPass,
            write: true,
            clear: true,
            width,
            height,
        }],
        dependencies: Vec::new(),

        width,
        height,
    };

    let pass = pass::create_render_pass(pass_desc).expect("Failed to create HDRI render pass.");

    let mut box_model = loader::load_device_model_from_obj(Path::new("data/models/box/box.obj"));
    pass::bind_technique_to_render_pass(&mut techs, &pass);
    pass::bind_device_model_to_render_pass(&mut box_model, &pass);
    if let Err(msg) = pass::is_render_pass_valid(&pass, &techs, &box_model) {
        panic!(msg);
    }

    (techs, pass, box_model)
}

fn create_attachments(width: u32, height: u32) -> Vec<pass::PassAttachment> {
    pass::create_pass_attachments(&vec![pass::PassAttachmentDescriptor {
        flavor: pass::PassAttachmentType::Color(math::zero_vec4()),
        source: pass::PassTextureSource::ThisPass,
        write: true,
        clear: true,
        width,
        height,
    }])
}
