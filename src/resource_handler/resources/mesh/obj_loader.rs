use std::{
    fs::File,
    io::{BufRead, BufReader},
    vec::Vec,
};

use ahash::{HashMap, HashMapExt};

use crate::{rendering::vulkan::buffers::Vertex, resource_handler::resources::mesh::Model};

fn parse_face_vertex(s: &str) -> [usize; 2] {
    let mut parts = s.split('/');
    let v = parts.next().unwrap().parse::<usize>().unwrap() - 1;
    let vt = parts.next().unwrap().parse::<usize>().unwrap() - 1;
    [v, vt]
}

pub fn load_obj(model_path: &str) -> Model {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut tex_coords: Vec<[f32; 2]> = Vec::new();
    let mut face_indices: Vec<[usize; 2]> = Vec::new();

    let obj_file = File::open(model_path).unwrap();
    let reader = BufReader::new(obj_file);

    for line_result in reader.lines() {
        if let Ok(line) = line_result {
            if let Some(coords) = line.strip_prefix("v ") {
                let nums: Vec<f32> = coords
                    .split_whitespace()
                    .map(|s| s.parse().unwrap())
                    .collect();
                positions.push([nums[0], nums[1], nums[2]]);
            } else if let Some(coords) = line.strip_prefix("vt") {
                let nums: Vec<f32> = coords
                    .split_whitespace()
                    .map(|s| s.parse().unwrap())
                    .collect();
                tex_coords.push([nums[0], 1.0 - nums[1]]);
            } else if let Some(coords) = line.strip_prefix("f ") {
                let mut elements = coords.split_whitespace();

                for _ in 0..3 {
                    face_indices.push(parse_face_vertex(elements.next().unwrap()));
                }
            }
        } else {
            break;
        }
    }

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let mut hash_map = HashMap::new();

    for index in face_indices {
        if let Some(new_index) = hash_map.get(&index) {
            indices.push(*new_index as u32);
        } else {
            hash_map.insert(index, vertices.len());
            indices.push(vertices.len() as u32);
            vertices.push(Vertex {
                pos: positions[index[0]],
                color: [1.0, 1.0, 1.0],
                tex_coord: tex_coords[index[1]],
            });
        }
    }

    Model { vertices, indices }
}
