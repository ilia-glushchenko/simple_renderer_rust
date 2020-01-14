extern crate stb_image;
extern crate tobj;
use crate::buffer;
use crate::log;
use crate::math;
use stb_image::image;
use std::mem::size_of;
use std::path::Path;
use std::vec::Vec;

type Vertices = Vec<math::Vec3f>;
type Indices = Vec<math::Vec1u>;
type Normals = Vec<math::Vec3f>;
type UVs = Vec<math::Vec2f>;

#[derive(Clone)]
pub struct MeshAttribute {
    pub name: String,
    pub dimensions: i32,
    pub stride: i32,
    pub data_type: u32,
}

#[derive(Clone)]
pub struct HostMesh {
    pub attributes: Vec<MeshAttribute>,
    pub material_index: u32,
    pub vertices: Vertices,
    pub normals: Normals,
    pub uvs: UVs,
    pub indices: Indices,
}

#[derive(Clone)]
pub struct HostTexture {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub struct HostMaterial {
    pub name: String,
    pub albedo_texture: Option<HostTexture>,
    pub normal_texture: Option<HostTexture>,
    pub bump_texture: Option<HostTexture>,
    pub metallic_texture: Option<HostTexture>,
    pub roughness_texture: Option<HostTexture>,
}

#[derive(Clone)]
pub struct HostModel {
    pub meshes: Vec<HostMesh>,
    pub materials: Vec<HostMaterial>,
}

#[derive(Clone)]
pub struct DeviceMesh {
    pub vao: u32,
    pub index_count: u32,
    pub attributes: Vec<MeshAttribute>,
    pub vbos: Vec<u32>,
    pub indices: u32,
    pub material_index: u32,
}

#[derive(Clone)]
pub struct DeviceTexture {
    pub name: String,
    pub handle: u32,
}

#[derive(Clone)]
pub struct SamplerProgramBinding {
    pub binding: u32,
    pub program: u32,
}

#[derive(Clone)]
pub struct Sampler2d {
    pub bindings: Vec<SamplerProgramBinding>,
    pub texture: DeviceTexture,
}

#[derive(Clone)]
pub struct DeviceMaterial {
    pub albedo_texture: Option<Sampler2d>,
    pub normal_texture: Option<Sampler2d>,
    pub bump_texture: Option<Sampler2d>,
    pub metallic_texture: Option<Sampler2d>,
    pub roughness_texture: Option<Sampler2d>,
}

#[derive(Clone)]
pub struct DeviceModel {
    pub meshes: Vec<DeviceMesh>,
    pub materials: Vec<DeviceMaterial>,
}

pub fn create_device_model(host_model: &HostModel) -> DeviceModel {
    let mut materials: Vec<DeviceMaterial> = Vec::new();
    for host_material in &host_model.materials {
        materials.push(buffer::create_device_material(host_material));
    }

    let mut meshes: Vec<DeviceMesh> = Vec::new();
    for host_mesh in &host_model.meshes {
        meshes.push(buffer::create_device_mesh(host_mesh));
    }

    DeviceModel { meshes, materials }
}

pub fn load_host_model_from_obj(file_path: &Path) -> HostModel {
    assert!(
        file_path.extension().unwrap() == "obj",
        "This function shall only load OBJ files.",
    );

    let message = format!("Failed to load obj file {}", file_path.to_str().unwrap()).to_string();
    let (raw_models, raw_materials) = tobj::load_obj(&file_path).expect(&message);

    let mut materials: Vec<HostMaterial> = Vec::new();
    for raw_material in &raw_materials {
        materials.push(create_host_material_from_tobj_material(
            &file_path.parent().unwrap_or(Path::new("")),
            raw_material,
        ));
    }

    let mut meshes: Vec<HostMesh> = Vec::new();
    for raw_model in &raw_models {
        meshes.push(create_host_mesh_from_tobj_mesh(raw_model));
    }

    HostModel { meshes, materials }
}

#[allow(dead_code)]
pub fn create_host_triangle_model() -> HostMesh {
    let vertices: Vec<math::Vec3f> = vec![
        math::Vec3 {
            x: -1.0f32,
            y: -1.0f32,
            z: 0.5f32,
        },
        math::Vec3 {
            x: 1.0f32,
            y: -1.0f32,
            z: 0.5f32,
        },
        math::Vec3 {
            x: 0.0f32,
            y: 1.0f32,
            z: 0.5f32,
        },
    ];
    let normals = vec![
        math::Vec3 {
            x: 0.0f32,
            y: 0.0f32,
            z: 1.0f32,
        },
        math::Vec3 {
            x: 0.0f32,
            y: 0.0f32,
            z: 1.0f32,
        },
        math::Vec3 {
            x: 0.0f32,
            y: 0.0f32,
            z: 1.0f32,
        },
    ];
    let uvs = vec![
        math::Vec2 {
            x: 0.0f32,
            y: 0.0f32,
        },
        math::Vec2 {
            x: 1.0f32,
            y: 0.0f32,
        },
        math::Vec2 {
            x: 0.5f32,
            y: 1.0f32,
        },
    ];
    let indices = vec![
        math::Vec1u { x: 0u32 },
        math::Vec1u { x: 1u32 },
        math::Vec1u { x: 2u32 },
    ];

    create_host_mesh(0, vertices, normals, uvs, indices)
}

fn create_host_mesh_from_tobj_mesh(raw_model: &tobj::Model) -> HostMesh {
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

    let mut vertices: Vertices = Vec::new();
    let mut normals: Normals = Vec::new();
    let mut uvs: UVs = Vec::new();
    let mut indices: Indices = Vec::new();

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

    create_host_mesh(
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
) -> HostMaterial {
    HostMaterial {
        name: raw_material.name.clone(),
        albedo_texture: create_host_texture(
            &folder_path.join(Path::new(&raw_material.diffuse_texture)),
        ),
        normal_texture: create_host_texture(
            &folder_path.join(Path::new(
                &raw_material
                    .unknown_param
                    .get(&"norm".to_string())
                    .unwrap_or(&"".to_string())
                    .to_string(),
            )),
        ),
        bump_texture: create_host_texture(
            &folder_path.join(Path::new(
                &raw_material
                    .unknown_param
                    .get(&"bump".to_string())
                    .unwrap_or(&"".to_string())
                    .to_string(),
            )),
        ),
        metallic_texture: create_host_texture(
            &folder_path.join(Path::new(
                &raw_material
                    .unknown_param
                    .get(&"map_Rm".to_string())
                    .unwrap_or(&"".to_string())
                    .to_string(),
            )),
        ),
        roughness_texture: create_host_texture(
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

fn create_host_texture(path: &Path) -> Option<HostTexture> {
    let load_result = image::load(path);

    if let image::LoadResult::ImageU8(image) = load_result {
        log::log_info(format!(
            "Loaded 8-bit texture: {}",
            path.to_str().unwrap_or_default()
        ));

        return Some(HostTexture {
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

fn create_host_mesh(
    material_index: u32,
    vertices: Vertices,
    normals: Normals,
    uvs: UVs,
    indices: Indices,
) -> HostMesh {
    HostMesh {
        attributes: create_model_attributes(&vertices, &normals, &uvs),
        material_index,
        vertices,
        normals,
        uvs,
        indices,
    }
}

#[allow(clippy::ptr_arg)]
fn create_model_attributes(
    vertices: &Vertices,
    normals: &Normals,
    uvs: &UVs,
) -> Vec<MeshAttribute> {
    assert!(!vertices.is_empty(), "Model must always have vertices.");

    let mut attributes: Vec<MeshAttribute> = Vec::new();

    if !vertices.is_empty() {
        attributes.push(create_model_attribute(&vertices, "aPosition".to_string()));
    }
    if !normals.is_empty() {
        attributes.push(create_model_attribute(&normals, "aNormal".to_string()));
    }
    if !uvs.is_empty() {
        attributes.push(create_model_attribute(&uvs, "aUV".to_string()));
    }

    attributes
}

fn create_model_attribute<T>(_: &[T], name: String) -> MeshAttribute
where
    T: math::VecDimensions<T>,
    T: math::VecGLTypeTrait,
{
    MeshAttribute {
        name,
        dimensions: T::DIMENSIONS as i32,
        stride: size_of::<T>() as i32,
        data_type: <T as math::VecGLTypeTrait>::GL_TYPE,
    }
}
