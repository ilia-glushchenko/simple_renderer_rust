extern crate stb_image;
extern crate tobj;
use crate::helper;
use crate::log;
use crate::math;
use crate::model;
use crate::shader;
use crate::tex;
use stb_image::image;
use std::fs;
use std::path::Path;

pub fn load_host_shader_program(
    descriptor: &shader::HostShaderProgramDescriptor,
) -> Result<shader::HostShaderProgram, String> {
    if descriptor.name.is_empty() {
        return Result::Err("Empty host shader program name".to_string());
    }

    let vert_shader_os_file_path = Path::new(&descriptor.vert_shader_file_path);
    if !vert_shader_os_file_path.exists() {
        return Result::Err(format!(
            "Shader file does not exists: {}",
            descriptor.vert_shader_file_path
        ));
    }
    let frag_shader_os_file_path = Path::new(&descriptor.frag_shader_file_path);
    if !frag_shader_os_file_path.exists() {
        return Result::Err(format!(
            "Shader file does not exists: {}",
            descriptor.frag_shader_file_path
        ));
    }

    let vert_shader_source = fs::read_to_string(vert_shader_os_file_path);
    if let Err(msg) = vert_shader_source {
        return Result::Err(format!("Failed to load shader code: {}", msg));
    }
    let vert_shader_source = vert_shader_source.unwrap();
    let frag_shader_source = fs::read_to_string(frag_shader_os_file_path);
    if let Err(msg) = frag_shader_source {
        return Result::Err(format!("Failed to load shader code: {}", msg));
    }
    let frag_shader_source = frag_shader_source.unwrap();

    Result::Ok(shader::HostShaderProgram {
        descriptor: descriptor.clone(),
        vert_shader_source,
        frag_shader_source,
    })
}

pub fn load_device_model_from_obj(path: &Path) -> model::DeviceModel {
    let host_model = load_host_model_from_obj(path);
    let device_model = model::create_device_model(&host_model);

    device_model
}

pub fn load_host_texture_from_file(path: &Path) -> Result<tex::HostTexture, String> {
    match image::load(path) {
        image::LoadResult::ImageU8(image) => {
            log::log_info(format!(
                "Loaded 8-bit texture: {}",
                path.to_str().unwrap_or_default()
            ));

            Ok(tex::HostTexture {
                name: String::from(
                    path.file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default(),
                ),
                width: image.width,
                height: image.height,
                depth: image.depth,
                data: tex::HostTextureData::UINT8(image.data),
            })
        }
        image::LoadResult::ImageF32(image) => {
            log::log_info(format!(
                "Loaded 32-bit texture: {}",
                path.to_str().unwrap_or_default()
            ));

            Ok(tex::HostTexture {
                name: String::from(
                    path.file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default(),
                ),
                width: image.width,
                height: image.height,
                depth: image.depth,
                data: tex::HostTextureData::FLOAT32(image.data),
            })
        }
        image::LoadResult::Error(msg) => {
            let msg = format!(
                "Failed to load texture: {}\nSTB message: {}",
                path.to_str().unwrap_or_default(),
                msg
            );
            log::log_error(msg.clone());
            Err(msg)
        }
    }
}

fn load_host_model_from_obj(file_path: &Path) -> model::HostModel {
    assert!(
        file_path.extension().unwrap() == "obj",
        "This function shall only load OBJ files.",
    );
    assert!(
        file_path.exists(),
        format!("File does not exist '{}'", file_path.to_str().unwrap())
    );

    let load_result = tobj::load_obj(&file_path);
    if let Err(error_type) = &load_result {
        panic!(format!(
            "Failed to load obj file '{}' '{}'",
            file_path.to_str().unwrap(),
            error_type
        ));
    }
    let (raw_models, raw_materials) = load_result.unwrap();

    let mut materials: Vec<model::HostMaterial> = Vec::new();
    for raw_material in &raw_materials {
        if let Some(folder) = file_path.parent() {
            materials.push(create_host_material_from_tobj_material(
                &folder,
                raw_material,
            ));
        } else {
            panic!("Failed to load model, folder path was invalid!");
        }
    }

    let mut meshes: Vec<model::HostMesh> = Vec::new();
    for raw_model in &raw_models {
        meshes.push(create_host_mesh_from_tobj_mesh(raw_model));
    }

    model::HostModel { meshes, materials }
}

fn create_host_mesh_from_tobj_mesh(raw_model: &tobj::Model) -> model::HostMesh {
    assert!(
        !raw_model.mesh.positions.is_empty(),
        "Model must have vertices."
    );
    assert!(
        !raw_model.mesh.indices.is_empty(),
        "Model must have indices."
    );
    assert!(
        raw_model.mesh.positions.len() == raw_model.mesh.normals.len()
            || raw_model.mesh.normals.is_empty(),
        "Every vertex must have a normal if there are normals."
    );
    assert!(
        raw_model.mesh.positions.len() / 3 == raw_model.mesh.texcoords.len() / 2
            || raw_model.mesh.texcoords.is_empty(),
        "Every vertex must have a uv if there are uvs."
    );

    let mut vertices = model::Vertices::new();
    let mut normals = model::Normals::new();
    let mut uvs = model::UVs::new();
    let mut indices = model::Indices::new();

    let mut i = 0;
    while i < raw_model.mesh.positions.len() {
        vertices.push(math::Vec3f {
            x: raw_model.mesh.positions[i],
            y: raw_model.mesh.positions[i + 1],
            z: raw_model.mesh.positions[i + 2],
        });
        i += 3;
    }
    let mut i = 0;
    while i < raw_model.mesh.normals.len() {
        normals.push(math::Vec3f {
            x: raw_model.mesh.normals[i],
            y: raw_model.mesh.normals[i + 1],
            z: raw_model.mesh.normals[i + 2],
        });
        i += 3;
    }
    let mut i = 0;
    while i < raw_model.mesh.texcoords.len() {
        uvs.push(math::Vec2f {
            x: raw_model.mesh.texcoords[i],
            y: raw_model.mesh.texcoords[i + 1],
        });
        i += 2;
    }
    let mut i = 0;
    while i < raw_model.mesh.indices.len() {
        indices.push(math::Vec1u {
            x: raw_model.mesh.indices[i],
        });
        i += 1;
    }

    let (tangents, bitangents) =
        helper::calculate_tangents_and_bitangents(&indices, &vertices, &uvs);

    model::create_host_mesh(
        raw_model
            .mesh
            .material_id
            .expect("Model should have a material"),
        vertices,
        normals,
        tangents,
        bitangents,
        uvs,
        indices,
    )
}

fn create_host_material_from_tobj_material(
    folder_path: &Path,
    raw_material: &tobj::Material,
) -> model::HostMaterial {
    let albedo_texture_path = folder_path.join(Path::new(&raw_material.diffuse_texture));
    let mut albedo_texture = None;
    if albedo_texture_path.is_file() {
        if let Ok(texture) = load_host_texture_from_file(&albedo_texture_path) {
            albedo_texture = Some(texture);
        }
    }

    let normal_texture_path = folder_path.join(Path::new(
        &raw_material
            .unknown_param
            .get(&"norm".to_string())
            .unwrap_or(&"".to_string())
            .to_string(),
    ));
    let mut normal_texture = None;
    if normal_texture_path.is_file() {
        if let Ok(texture) = load_host_texture_from_file(&normal_texture_path) {
            normal_texture = Some(texture);
        }
    }

    let bump_texture_path = folder_path.join(Path::new(
        &raw_material
            .unknown_param
            .get(&"bump".to_string())
            .unwrap_or(&"".to_string())
            .to_string(),
    ));
    let mut bump_texture = None;
    if bump_texture_path.is_file() {
        if let Ok(texture) = load_host_texture_from_file(&bump_texture_path) {
            bump_texture = Some(texture);
        }
    }

    let metallic_texture_path = folder_path.join(Path::new(
        &raw_material
            .unknown_param
            .get(&"map_Rm".to_string())
            .unwrap_or(&"".to_string())
            .to_string(),
    ));
    let mut metallic_texture = None;
    if metallic_texture_path.is_file() {
        if let Ok(texture) = load_host_texture_from_file(&metallic_texture_path) {
            metallic_texture = Some(texture);
        }
    }

    let roughness_texture_path = folder_path.join(Path::new(
        &raw_material
            .unknown_param
            .get(&"map_Pr".to_string())
            .unwrap_or(&"".to_string())
            .to_string(),
    ));
    let mut roughness_texture = None;
    if roughness_texture_path.is_file() {
        if let Ok(texture) = load_host_texture_from_file(&roughness_texture_path) {
            roughness_texture = Some(texture);
        }
    }

    model::HostMaterial {
        name: raw_material.name.clone(),

        albedo_available: albedo_texture.is_some(),
        normal_available: normal_texture.is_some(),
        bump_available: bump_texture.is_some(),
        metallic_available: metallic_texture.is_some(),
        roughness_available: roughness_texture.is_some(),

        albedo_texture: albedo_texture,
        normal_texture: normal_texture,
        bump_texture: bump_texture,
        metallic_texture: metallic_texture,
        roughness_texture: roughness_texture,

        //ToDo: Those are default values, need to replace them with
        //values from the actual raw materials
        scalar_albedo: math::Vec3f {
            x: 1.,
            y: 1.,
            z: 1.,
        },
        scalar_roughness: math::Vec1f { x: 1. },
        scalar_metalness: math::Vec1f { x: 1. },
    }
}
