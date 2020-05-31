use crate::material;
use crate::math;
use crate::mesh;
use crate::model;
use std::rc::Rc;

pub fn calculate_tangents_and_bitangents(
    indices: &mesh::Indices,
    vertices: &mesh::Vertices,
    uvs: &mesh::UVs,
) -> (mesh::Tangents, mesh::Bitangents) {
    let mut tangents = mesh::Tangents::new();
    let mut bitangents = mesh::Bitangents::new();

    for i in (0..indices.len()).step_by(3) {
        let vert0 = &vertices[indices[i + 0].x as usize];
        let vert1 = &vertices[indices[i + 1].x as usize];
        let vert2 = &vertices[indices[i + 2].x as usize];

        let uv0 = &uvs[indices[i + 0].x as usize];
        let uv1 = &uvs[indices[i + 1].x as usize];
        let uv2 = &uvs[indices[i + 2].x as usize];

        let delta_pos1 = *vert1 - *vert0;
        let delta_pos2 = *vert2 - *vert0;

        let delta_uv1 = *uv1 - *uv0;
        let delta_uv2 = *uv2 - *uv0;

        let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
        let tangent =
            math::normalize_vec3((delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r);
        let bitangent =
            math::normalize_vec3((delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r);

        // if tangent.x.is_nan() || tangent.y.is_nan() || tangent.z.is_nan() {
        //     let pass = 0;
        // }

        // if bitangent.x.is_nan() || bitangent.y.is_nan() || bitangent.z.is_nan() {
        //     let pass = 0;
        // }

        // assert!(
        //     !tangent.x.is_nan() && !tangent.y.is_nan() && !tangent.z.is_nan(),
        //     "Nan tangent!"
        // );
        // assert!(
        //     !bitangent.x.is_nan() && !bitangent.y.is_nan() && !bitangent.z.is_nan(),
        //     "Nan bitangent!"
        // );

        tangents.push(tangent);
        tangents.push(tangent);
        tangents.push(tangent);
        bitangents.push(bitangent);
        bitangents.push(bitangent);
        bitangents.push(bitangent);
    }

    return (tangents, bitangents);
}

#[allow(dead_code)]
pub fn create_host_triangle_model() -> mesh::HostMesh {
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

    let (tangents, bitangents) = calculate_tangents_and_bitangents(&indices, &vertices, &uvs);

    model::create_host_mesh(0, vertices, normals, tangents, bitangents, uvs, indices)
}

pub fn create_full_screen_triangle_host_mesh() -> mesh::HostMesh {
    let vertices: Vec<math::Vec3f> = vec![
        math::Vec3 {
            x: -1.0f32,
            y: -1.0f32,
            z: 0.5f32,
        },
        math::Vec3 {
            x: 3.0f32,
            y: -1.0f32,
            z: 0.5f32,
        },
        math::Vec3 {
            x: -1.0f32,
            y: 3.0f32,
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
            x: 2.0f32,
            y: 0.0f32,
        },
        math::Vec2 {
            x: 0.0f32,
            y: 2.0f32,
        },
    ];
    let indices = vec![
        math::Vec1u { x: 0u32 },
        math::Vec1u { x: 1u32 },
        math::Vec1u { x: 2u32 },
    ];

    let (tangents, bitangents) = calculate_tangents_and_bitangents(&indices, &vertices, &uvs);

    model::create_host_mesh(0, vertices, normals, tangents, bitangents, uvs, indices)
}

pub fn create_full_screen_triangle_model() -> model::DeviceModel {
    model::create_device_model(&model::HostModel {
        meshes: Rc::new(vec![create_full_screen_triangle_host_mesh()]),
        materials: Rc::new(vec![material::HostMaterial::empty()]),
    })
}
