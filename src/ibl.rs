use crate::helper;
use crate::math;
use crate::model;
use crate::pass;
use crate::shader;
use crate::tech;
use crate::techniques;
use crate::tex;
use std::f32;
use std::ffi::c_void;
use std::ptr::null;
use std::rc::Rc;

pub fn create_specular_cube_map_texture(
    host_texture: &tex::HostTexture,
    box_model: &mut model::DeviceModel,
) -> Rc<tex::DeviceTexture> {
    let width: u32 = 2048;
    let height: u32 = 2048;

    let (mut techs, mut pass) = create_hdri_2_cube_map_pass(host_texture, box_model, width, height);

    let fbos = draw_cubemap(&mut techs, &mut pass, &box_model);

    let desc = tex::create_spherical_hdri_texture_descriptor(&host_texture);
    let cubemap = create_cubemap(&desc, width, height, &fbos);

    cleanup_cubemap_fbos(fbos, &mut techs, pass, box_model);
    tech::delete_techniques(techs);

    cubemap
}

pub fn create_diffuse_cube_map_texture(
    cube_map: Rc<tex::DeviceTexture>,
    box_model: &mut model::DeviceModel,
) -> Rc<tex::DeviceTexture> {
    let width: u32 = 32;
    let height: u32 = 32;

    let (mut techs, mut pass) =
        create_diffuse_cubemap_convolution_pass(cube_map, box_model, width, height);

    let fbos = draw_cubemap(&mut techs, &mut pass, &box_model);

    let desc = tex::create_color_attachment_device_texture_descriptor();
    let cubemap = create_cubemap(&desc, width, height, &fbos);

    cleanup_cubemap_fbos(fbos, &mut techs, pass, box_model);

    cubemap
}

pub fn create_prefiltered_environment_map(
    cube_map: Rc<tex::DeviceTexture>,
    box_model: &mut model::DeviceModel,
) -> Rc<tex::DeviceTexture> {
    let width = 128;
    let height = 128;

    let desc = tex::create_prefiltered_env_map_texture_descriptor();
    let map = create_empty_cubemap(&desc, width, height);

    let (mut techs, mut pass) =
        create_prefiltered_environment_map_pass(cube_map, map.clone(), box_model, width, height);

    let views = [
        math::y_rotation_mat4x4(-f32::consts::PI),
        math::identity_mat4x4(),
        math::x_rotation_mat4x4(f32::consts::PI / 2.)
            * math::y_rotation_mat4x4(f32::consts::PI / 2.),
        math::x_rotation_mat4x4(-f32::consts::PI / 2.)
            * math::y_rotation_mat4x4(f32::consts::PI / 2.),
        math::y_rotation_mat4x4(f32::consts::PI / 2.),
        math::y_rotation_mat4x4(-f32::consts::PI / 2.),
    ];

    for mip_level in 0..8 {
        let roughness = mip_level as f32 / 7_f32;

        let width = (width as f32 * 0.5_f32.powf(mip_level as f32)) as u32;
        let height = (height as f32 * 0.5_f32.powf(mip_level as f32)) as u32;

        for face_index in 0..6 {
            pass.recreate_attachments(&create_env_map_attachment_descriptors(
                map.clone(),
                (gl::TEXTURE_CUBE_MAP_POSITIVE_X as usize + face_index) as gl::types::GLenum,
                width,
                height,
                mip_level,
            ))
            .unwrap();
            pass.width = width;
            pass.height = height;

            techniques::ibl::prefiltered_envirnoment_map::update(
                techs.get_mut(&tech::Techniques::IBL).unwrap(),
                views[face_index],
                roughness,
            );

            pass::execute_render_pass(&pass, &techs, &box_model);
        }
    }

    map
}

pub fn create_brdf_lut() -> Rc<tex::DeviceTexture> {
    let width: u32 = 128;
    let height: u32 = 128;

    let (mut techs, mut pass, mut model) = create_brdf_integration_map_pass(width, height);
    pass::execute_render_pass(&pass, &techs, &model);

    let result = pass.attachments[0].texture.clone();
    pass.attachments.clear();

    pass::unbind_techniques_from_render_pass(&mut techs, pass.program.handle);
    pass::unbind_device_model_from_render_pass(&mut model, pass.program.handle);
    pass::unbind_techniques_from_render_pass(&mut techs, pass.program.handle);
    pass::delete_render_pass(&mut pass);
    model::delete_device_model(model);

    result
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
) -> Rc<tex::DeviceTexture> {
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

        if desc.use_mipmaps {
            gl::GenerateMipmap(gl::TEXTURE_CUBE_MAP);
        }

        gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);
    }

    Rc::new(tex::DeviceTexture {
        handle,
        target: gl::TEXTURE_CUBE_MAP,
    })
}

fn create_empty_cubemap(
    desc: &tex::DeviceTextureDescriptor,
    width: u32,
    height: u32,
) -> Rc<tex::DeviceTexture> {
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

        if desc.use_mipmaps {
            gl::GenerateMipmap(gl::TEXTURE_CUBE_MAP);
        }

        gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);
    }

    Rc::new(tex::DeviceTexture {
        handle,
        target: gl::TEXTURE_CUBE_MAP,
    })
}

fn cleanup_cubemap_fbos(
    mut fbos: CubeMapFbos,
    techs: &mut tech::TechniqueMap,
    mut pass: pass::Pass,
    box_model: &mut model::DeviceModel,
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

    pass::unbind_techniques_from_render_pass(techs, pass.program.handle);
    pass::unbind_device_model_from_render_pass(box_model, pass.program.handle);
    pass::unbind_techniques_from_render_pass(techs, pass.program.handle);
    pass::delete_render_pass(&mut pass);
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
    box_model: &mut model::DeviceModel,
    width: u32,
    height: u32,
) -> (tech::TechniqueMap, pass::Pass) {
    let mut techs = tech::TechniqueMap::new();
    techs.insert(
        tech::Techniques::IBL,
        techniques::ibl::hdri2cube::create(
            math::perspective_projection_mat4x4(f32::consts::PI / 2.0, 1., 0.98, 1.01),
            tex::create_device_texture(
                &host_texture,
                &tex::create_spherical_hdri_texture_descriptor(&host_texture),
            ),
        ),
    );

    let pass_desc = pass::PassDescriptor {
        name: "HDRI 2 Cube".to_string(),
        program: shader::HostShaderProgramDescriptor {
            name: "HDIR 2 Cube".to_string(),
            vert_shader_file_path: "shaders/ibl/cube_map_pass_through.vert".to_string(),
            frag_shader_file_path: "shaders/ibl/hdri2cube.frag".to_string(),
        },
        techniques: vec![tech::Techniques::IBL],

        attachments: vec![pass::PassAttachmentDescriptor {
            texture_desc: tex::create_color_attachment_device_texture_descriptor(),
            flavor: pass::PassAttachmentType::Color(math::zero_vec4()),
            source: pass::PassTextureSource::ThisPass,
            textarget: gl::TEXTURE_2D,
            write: true,
            clear: true,
            width,
            height,
            mip_level: 0,
        }],
        dependencies: Vec::new(),

        width,
        height,
    };

    let pass = pass::create_render_pass(pass_desc).expect("Failed to create HDRI render pass.");

    pass::bind_techniques_to_render_pass(&mut techs, &pass);
    pass::bind_device_model_to_render_pass(box_model, &pass);
    if let Err(msg) = pass::is_render_pass_valid(&pass, &techs, &box_model) {
        panic!(msg);
    }

    (techs, pass)
}

fn create_diffuse_cubemap_convolution_pass(
    cube_map: Rc<tex::DeviceTexture>,
    box_model: &mut model::DeviceModel,
    width: u32,
    height: u32,
) -> (tech::TechniqueMap, pass::Pass) {
    let mut techs = tech::TechniqueMap::new();
    techs.insert(
        tech::Techniques::IBL,
        techniques::ibl::diffuse_cubemap_convolution::create(
            math::perspective_projection_mat4x4(f32::consts::PI / 2.0, 1., 0.98, 1.01),
            cube_map,
        ),
    );

    let pass_desc = pass::PassDescriptor {
        name: "Cubemap Convolution".to_string(),
        program: shader::HostShaderProgramDescriptor {
            name: "Cubemap Convolution".to_string(),
            vert_shader_file_path: "shaders/ibl/cube_map_pass_through.vert".to_string(),
            frag_shader_file_path: "shaders/ibl/diffuse_cube_map_convolution.frag".to_string(),
        },
        techniques: vec![tech::Techniques::IBL],

        attachments: vec![pass::PassAttachmentDescriptor {
            texture_desc: tex::create_color_attachment_device_texture_descriptor(),
            flavor: pass::PassAttachmentType::Color(math::zero_vec4()),
            source: pass::PassTextureSource::ThisPass,
            textarget: gl::TEXTURE_2D,
            write: true,
            clear: true,
            width,
            height,
            mip_level: 0,
        }],
        dependencies: Vec::new(),

        width,
        height,
    };

    let pass = pass::create_render_pass(pass_desc).expect("Failed to create HDRI render pass.");

    pass::bind_techniques_to_render_pass(&mut techs, &pass);
    pass::bind_device_model_to_render_pass(box_model, &pass);
    if let Err(msg) = pass::is_render_pass_valid(&pass, &techs, &box_model) {
        panic!(msg);
    }

    (techs, pass)
}

fn create_brdf_integration_map_pass(
    width: u32,
    height: u32,
) -> (tech::TechniqueMap, pass::Pass, model::DeviceModel) {
    let mut techs = tech::TechniqueMap::new();
    techs.insert(
        tech::Techniques::IBL,
        techniques::ibl::brdf_integration_map::create(),
    );

    let mut texture_desc = tex::create_color_attachment_device_texture_descriptor();
    texture_desc.use_mipmaps = false;

    let pass_desc = pass::PassDescriptor {
        name: "BRDF Integration Map".to_string(),
        program: shader::HostShaderProgramDescriptor {
            name: "BRDF Integration Map".to_string(),
            vert_shader_file_path: "shaders/pass_through.vert".to_string(),
            frag_shader_file_path: "shaders/ibl/brdf_integration_map.frag".to_string(),
        },
        techniques: vec![tech::Techniques::IBL],

        attachments: vec![pass::PassAttachmentDescriptor {
            texture_desc: texture_desc,
            flavor: pass::PassAttachmentType::Color(math::zero_vec4()),
            source: pass::PassTextureSource::ThisPass,
            textarget: gl::TEXTURE_2D,
            write: true,
            clear: true,
            width,
            height,
            mip_level: 0,
        }],
        dependencies: Vec::new(),

        width,
        height,
    };

    let pass =
        pass::create_render_pass(pass_desc).expect("Failed to create BRDF integration pass.");
    let mut fullscreen_model = helper::create_full_screen_triangle_model();
    pass::bind_techniques_to_render_pass(&mut techs, &pass);
    pass::bind_device_model_to_render_pass(&mut fullscreen_model, &pass);
    if let Err(msg) = pass::is_render_pass_valid(&pass, &techs, &fullscreen_model) {
        panic!(msg);
    }

    (techs, pass, fullscreen_model)
}

fn create_prefiltered_environment_map_pass(
    source_cube_map: Rc<tex::DeviceTexture>,
    target_cube_map: Rc<tex::DeviceTexture>,
    box_model: &mut model::DeviceModel,
    width: u32,
    height: u32,
) -> (tech::TechniqueMap, pass::Pass) {
    let mut techs = tech::TechniqueMap::new();
    techs.insert(
        tech::Techniques::IBL,
        techniques::ibl::prefiltered_envirnoment_map::create(
            math::perspective_projection_mat4x4(f32::consts::PI / 2.0, 1., 0.98, 1.01),
            source_cube_map,
            0_f32,
        ),
    );

    let pass_desc = pass::PassDescriptor {
        name: "Cubemap Convolution".to_string(),
        program: shader::HostShaderProgramDescriptor {
            name: "Cubemap Convolution".to_string(),
            vert_shader_file_path: "shaders/ibl/cube_map_pass_through.vert".to_string(),
            frag_shader_file_path: "shaders/ibl/prefiltered_environment_map.frag".to_string(),
        },
        techniques: vec![tech::Techniques::IBL],

        attachments: create_env_map_attachment_descriptors(
            target_cube_map.clone(),
            gl::TEXTURE_CUBE_MAP_POSITIVE_X,
            width,
            height,
            0,
        ),
        dependencies: Vec::new(),

        width,
        height,
    };

    let pass = pass::create_render_pass(pass_desc).expect("Failed to create HDRI render pass.");

    pass::bind_techniques_to_render_pass(&mut techs, &pass);
    pass::bind_device_model_to_render_pass(box_model, &pass);
    if let Err(msg) = pass::is_render_pass_valid(&pass, &techs, &box_model) {
        panic!(msg);
    }

    (techs, pass)
}

fn create_env_map_attachment_descriptors(
    cube_map: Rc<tex::DeviceTexture>,
    textarget: gl::types::GLenum,
    width: u32,
    height: u32,
    mip_level: i32,
) -> Vec<pass::PassAttachmentDescriptor> {
    vec![pass::PassAttachmentDescriptor {
        texture_desc: tex::create_color_attachment_device_texture_descriptor(),
        flavor: pass::PassAttachmentType::Color(math::zero_vec4()),
        source: pass::PassTextureSource::FreeTexture(cube_map),
        textarget,
        write: true,
        clear: true,
        width,
        height,
        mip_level,
    }]
}

fn create_attachments(width: u32, height: u32) -> Vec<pass::PassAttachment> {
    pass::create_pass_attachments(&vec![pass::PassAttachmentDescriptor {
        texture_desc: tex::create_color_attachment_device_texture_descriptor(),
        flavor: pass::PassAttachmentType::Color(math::zero_vec4()),
        source: pass::PassTextureSource::ThisPass,
        textarget: gl::TEXTURE_2D,
        write: true,
        clear: true,
        width,
        height,
        mip_level: 0,
    }])
}
