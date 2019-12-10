extern crate tobj;
use crate::math;
use std::mem::size_of;
use std::vec::Vec;

#[derive(Clone)]
pub struct HostModel {
    pub attributes: Vec<ModelAttribute>,
    pub vertices: Vec<math::Vec3f>,
    pub normals: Vec<math::Vec3f>,
    pub uvs: Vec<math::Vec2f>,
    pub indices: Vec<math::Vec1u>,
    pub materials: Vec<math::Vec1u>,
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

// pub fn load_host_model(file_path: &Path) -> HostModel {
//     let (models, materials) =
//         tobj::load_obj(&Path::new(file_path)).expect("Failed to load box.obj file");

//     let mut vertices: Vec<math::Vec3f> = Vec::new();
//     let mut normals: Vec<math::Vec3f> = Vec::new();
//     let mut uvs: Vec<math::Vec2f> = Vec::new();
//     let mut indices: Vec<math::Vec1i> = Vec::new();
//     let mut materials: Vec<math::Vec1i> = Vec::new();

//     HostModel {
//         vertices,
//         normals,
//         uvs,
//         indices,
//         materials,
//     }
// }

pub fn create_model_attribute<T>(_: &[T], name: String) -> ModelAttribute
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

    HostModel {
        attributes: vec![
            create_model_attribute(&vertices, "aPosition".to_string()),
            create_model_attribute(&normals, "aNormal".to_string()),
            create_model_attribute(&uvs, "aUV".to_string()),
            create_model_attribute(&materials, "aMaterial".to_string()),
        ],
        vertices,
        normals,
        uvs,
        indices,
        materials,
    }
}
