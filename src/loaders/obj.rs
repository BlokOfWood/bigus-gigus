use std::{
    fs::File,
    io::{BufRead, BufReader},
    vec::Vec,
};

use crate::vulkan::buffers::Vertex;

pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Model {
    fn parse_face_vertex(s: &str) -> [usize; 2] {
        let mut parts = s.split('/');
        let v = parts.next().unwrap().parse::<usize>().unwrap() - 1;
        let vt = parts.next().unwrap().parse::<usize>().unwrap() - 1;
        [v, vt]
    }

    pub fn load_obj(model_path: &str) -> Self {
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
                        face_indices.push(Model::parse_face_vertex(elements.next().unwrap()));
                    }
                }
            } else {
                break;
            }
        }

        let mut indices = Vec::new();

        let vertices = face_indices
            .iter()
            .map(|face_index| {
                indices.push(indices.len() as u32);

                Vertex {
                    pos: positions[face_index[0]],
                    tex_coord: tex_coords[face_index[1]],
                    color: [0.0, 0.0, 0.0],
                }
            })
            .collect();

        Model { vertices, indices }
    }
}
