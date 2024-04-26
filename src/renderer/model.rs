use cgmath::{InnerSpace, Vector3};

use crate::renderer::triangle::Triangle;

use super::triangle::{Model, ModelArrayBuffer};

pub fn load_model(
    file_name: &str,
    ident: &str,
    model_buffer: &mut ModelArrayBuffer,
) {
    let (models, _materials) =
        tobj::load_obj(
            file_name,
            &tobj::LoadOptions::default()
        )
        .expect("Failed to OBJ load file");

    let mut model = Model::new(ident);

    for m in models {
        let mesh_offset = model.triangle_count;
        let normal_offset = model.normal_count;

        log::info!("normals: {}, positions: {}, indices: {}, normal_indices: {}, ", m.mesh.normals.len(), m.mesh.positions.len(), m.mesh.indices.len(), m.mesh.normal_indices.len());

        for i in 0..m.mesh.normals.len() / 3 {
            let p1 = m.mesh.normals[i * 3];
            let p2 = m.mesh.normals[i * 3 + 1];
            let p3 = m.mesh.normals[i * 3 + 2];

            model.add_normal([p1, p2, p3, 0.0]);
        }

        for i in 0..m.mesh.positions.len() / 3 {
            let p1 = m.mesh.positions[i * 3];
            let p2 = m.mesh.positions[i * 3 + 1] * -1.0;
            let p3 = m.mesh.positions[i * 3 + 2];

            model.add_vertex([p1, p2, p3, 0.0]);
        }

        for i in 0..m.mesh.indices.len() / 3 {
            let p1 = m.mesh.indices[i * 3] as i32;
            let p2 = m.mesh.indices[i * 3 + 1] as i32;
            let p3 = m.mesh.indices[i * 3 + 2] as i32;

            let (n1, n2, n3) = if !m.mesh.normal_indices.is_empty() {
                (
                    m.mesh.normal_indices[i * 3] as i32,
                    m.mesh.normal_indices[i * 3 + 1] as i32,
                    m.mesh.normal_indices[i * 3 + 2] as i32
                )
            } else {
                let p1 = model.points[p1 as usize];
                let p2 = model.points[p2 as usize];
                let p3 = model.points[p3 as usize];

                let a = Vector3::new(p1[0], p1[1], p1[2]);
                let b = Vector3::new(p2[0], p2[1], p2[2]);
                let c = Vector3::new(p3[0], p3[1], p3[2]);

                let dir = Vector3::cross(b - a, c - a).normalize();
                let index = model.normal_count;
                model.add_normal([dir.x, dir.y, dir.z, 0.0]);
                (index, index, index)
            };


            let triangle = Triangle {
                p1: p1 + mesh_offset, 
                p2: p2 + mesh_offset, 
                p3: p3 + mesh_offset,
                n1: n1 + normal_offset, 
                n2: n2 + normal_offset, 
                n3: n3 + normal_offset
            };

            model.add_triangle(triangle);
        }

        model.build_bvh();
        model_buffer.insert(model);
    }
}
