extern crate tobj;
use crate::math;
use std::mem::size_of;
use std::path::Path;
use std::vec::Vec;

type Vertices = Vec<math::Vec3f>;
type Indices = Vec<math::Vec1u>;
type Normals = Vec<math::Vec3f>;
type UVs = Vec<math::Vec2f>;
type Materials = Vec<math::Vec1u>;

#[derive(Clone)]
pub struct HostModel {
    pub attributes: Vec<ModelAttribute>,
    pub vertices: Vertices,
    pub normals: Normals,
    pub uvs: UVs,
    pub indices: Indices,
    pub materials: Materials,
}

#[derive(Clone)]
pub struct DeviceModel {
    pub vao: u32,
    pub index_count: i32,
    pub attributes: Vec<ModelAttribute>,
    pub vbos: Vec<u32>,
    pub indices: u32,
}

#[derive(Clone)]
pub struct ModelAttribute {
    pub name: String,
    pub dimensions: i32,
    pub stride: i32,
    pub data_type: u32,
}

pub fn load_host_model_from_obj(file_path: &Path) -> Vec<HostModel> {
    assert!(
        file_path.extension().unwrap() == "obj",
        "This function shall only load OBJ files.",
    );

    let (raw_models, _raw_materials) =
        tobj::load_obj(&Path::new(file_path)).expect("Failed to load box.obj file");

    let mut models: Vec<HostModel> = Vec::new();

    for model in &raw_models {
        models.push(create_host_model_from_tobj_model(model));
    }

    models
}

fn create_host_model_from_tobj_model(raw_model: &tobj::Model) -> HostModel {
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
    let mut materials: Materials = Vec::new();

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
    if let Some(material) = raw_model.mesh.material_id {
        materials = vec![math::Vec1u { x: material as u32 }; vertices.len()];
    }

    create_host_model(vertices, normals, uvs, materials, indices)
}

#[allow(dead_code)]
pub fn create_host_triangle_model() -> HostModel {
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
    let materials = vec![
        math::Vec1u { x: 0u32 },
        math::Vec1u { x: 0u32 },
        math::Vec1u { x: 0u32 },
    ];

    create_host_model(vertices, normals, uvs, materials, indices)
}

fn create_host_model(
    vertices: Vertices,
    normals: Normals,
    uvs: UVs,
    materials: Materials,
    indices: Indices,
) -> HostModel {
    HostModel {
        attributes: create_model_attributes(&vertices, &normals, &uvs, &materials),
        vertices,
        normals,
        uvs,
        indices,
        materials,
    }
}

#[allow(clippy::ptr_arg)]
fn create_model_attributes(
    vertices: &Vertices,
    normals: &Normals,
    uvs: &UVs,
    materials: &Materials,
) -> Vec<ModelAttribute> {
    assert!(!vertices.is_empty(), "Model must always have vertices.");

    let mut attributes: Vec<ModelAttribute> = Vec::new();

    if !vertices.is_empty() {
        attributes.push(create_model_attribute(&vertices, "aPosition".to_string()));
    }
    if !normals.is_empty() {
        attributes.push(create_model_attribute(&normals, "aNormal".to_string()));
    }
    if !uvs.is_empty() {
        attributes.push(create_model_attribute(&uvs, "aUV".to_string()));
    }
    if !materials.is_empty() {
        attributes.push(create_model_attribute(&materials, "aMaterial".to_string()));
    }

    attributes
}

fn create_model_attribute<T>(_: &[T], name: String) -> ModelAttribute
where
    T: math::VecDimensions<T>,
    T: math::VecGLTypeTrait,
{
    ModelAttribute {
        name,
        dimensions: T::DIMENSIONS as i32,
        stride: size_of::<T>() as i32,
        data_type: <T as math::VecGLTypeTrait>::GL_TYPE,
    }
}
