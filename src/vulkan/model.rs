use tobj::{load_obj, LoadOptions, GPU_LOAD_OPTIONS};

use crate::vulkan::buffers::Vertex;

pub fn import_model(model_path: &str) -> (Vec<Vertex>, Vec<u32>) {
    let result = load_obj(
        model_path,
        &LoadOptions {
            ignore_lines: false,
            ignore_points: false,
            single_index: true,
            triangulate: true,
        },
    )
    .unwrap();

    let (models, _) = result;

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indicies: Vec<u32> = Vec::new();

    for model in models {
        let mesh = model.mesh;
        let mesh_pos = mesh.positions;
        let mesh_tex_coord = mesh.texcoords;
        let mut mesh_indicies = mesh.indices;

        for index in mesh_indicies {
            let idx = index as usize;

            let new_vertex = Vertex {
                pos: [
                    mesh_pos[3 * idx],
                    mesh_pos[3 * idx + 1],
                    mesh_pos[3 * idx + 2],
                ],
                color: [0.0, 0.0, 0.0],
                tex_coord: [mesh_tex_coord[2 * idx], 1.0 - mesh_tex_coord[2 * idx + 1]],
            };

            vertices.push(new_vertex);
            indicies.push(indicies.len() as u32);
        }
    }

    return (vertices, indicies);
}
