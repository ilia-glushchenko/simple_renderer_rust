extern crate stb_image;
extern crate tobj;
use crate::log;
use crate::math;
use crate::model;
use crate::shader;
use crate::texture;
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

pub fn load_host_model_from_obj(file_path: &Path) -> model::HostModel {
    assert!(
        file_path.extension().unwrap() == "obj",
        "This function shall only load OBJ files.",
    );

    let message = format!("Failed to load obj file {}", file_path.to_str().unwrap()).to_string();
    let (raw_models, raw_materials) = tobj::load_obj(&file_path).expect(&message);

    let mut materials: Vec<model::HostMaterial> = Vec::new();
    for raw_material in &raw_materials {
        materials.push(create_host_material_from_tobj_material(
            &file_path.parent().unwrap_or(Path::new("")),
            raw_material,
        ));
    }

    let mut meshes: Vec<model::HostMesh> = Vec::new();
    for raw_model in &raw_models {
        meshes.push(create_host_mesh_from_tobj_mesh(raw_model));
    }

    // {
    //     meshes.first_mut().unwrap().material_index = 0;
    //     materials = vec![materials[meshes.first().unwrap().material_index as usize].clone()];
    //     meshes = vec![meshes.first().unwrap().clone()];
    // }

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

    let mut vertices: model::Vertices = Vec::new();
    let mut normals: model::Normals = Vec::new();
    let mut uvs: model::UVs = Vec::new();
    let mut indices: model::Indices = Vec::new();

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

    model::create_host_mesh(
        raw_model
            .mesh
            .material_id
            .expect("Model should have a material") as u32,
        vertices,
        normals,
        uvs,
        indices,
    )
}

fn create_host_material_from_tobj_material(
    folder_path: &Path,
    raw_material: &tobj::Material,
) -> model::HostMaterial {
    model::HostMaterial {
        name: raw_material.name.clone(),
        albedo_texture: load_host_texture_from_file(
            &folder_path.join(Path::new(&raw_material.diffuse_texture)),
        ),
        normal_texture: load_host_texture_from_file(
            &folder_path.join(Path::new(
                &raw_material
                    .unknown_param
                    .get(&"norm".to_string())
                    .unwrap_or(&"".to_string())
                    .to_string(),
            )),
        ),
        bump_texture: load_host_texture_from_file(
            &folder_path.join(Path::new(
                &raw_material
                    .unknown_param
                    .get(&"bump".to_string())
                    .unwrap_or(&"".to_string())
                    .to_string(),
            )),
        ),
        metallic_texture: load_host_texture_from_file(
            &folder_path.join(Path::new(
                &raw_material
                    .unknown_param
                    .get(&"map_Rm".to_string())
                    .unwrap_or(&"".to_string())
                    .to_string(),
            )),
        ),
        roughness_texture: load_host_texture_from_file(
            &folder_path.join(Path::new(
                &raw_material
                    .unknown_param
                    .get(&"map_Pr".to_string())
                    .unwrap_or(&"".to_string())
                    .to_string(),
            )),
        ),
    }
}

fn load_host_texture_from_file(path: &Path) -> Option<texture::HostTexture> {
    let load_result = image::load(path);

    if let image::LoadResult::ImageU8(image) = load_result {
        log::log_info(format!(
            "Loaded 8-bit texture: {}",
            path.to_str().unwrap_or_default()
        ));

        return Some(texture::HostTexture {
            name: String::from(
                path.file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default(),
            ),
            width: image.width,
            height: image.height,
            depth: image.depth,
            data: image.data,
        });
    }

    if let image::LoadResult::ImageF32(_image) = &load_result {
        log::log_warning(format!(
            "Loaded 32-bit texture: {}",
            path.to_str().unwrap_or_default()
        ));
    }

    if let image::LoadResult::Error(msg) = &load_result {
        if path.is_file() {
            log::log_error(format!(
                "Failed to load texture: {} \nSTB message: {}",
                path.to_str().unwrap_or_default(),
                msg
            ));
        }
    }

    None
}
