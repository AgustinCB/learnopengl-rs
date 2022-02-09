use nalgebra::{ArrayStorage, Matrix, Matrix4, Rotation3, Scale3, Translation3, U1, Vector2, Vector3};
use russimp::texture::TextureType;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use crate::buffer::Buffer;
use crate::ecs::systems::input::InputType;
use crate::texture::Texture;
use crate::vertex_array::VertexArray;

pub struct Input {
    pub input_types: Vec<InputType>,
    pub(crate) events: Vec<Event>,
}

impl Input {
    pub fn new(input_types: Vec<InputType>) -> Input {
        Input {
            input_types,
            events: vec![],
        }
    }
}

pub struct FpsCamera {
    pub camera_speed: f32,
}

pub struct QuitControl {
    pub quit_keycode: Keycode,
}

#[derive(Clone, Debug)]
pub struct TextureInfo {
    pub id: usize,
    pub texture_type: TextureType,
    pub width: usize,
    pub height: usize,
    pub path: String,
}

pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Rotation3<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        let t = Translation3::from(self.position);
        let s = Scale3::from(self.scale);
        t.to_homogeneous() * self.rotation.to_homogeneous() * s.to_homogeneous()
    }
}

pub struct Velocity(pub Vector3<f32>);
#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vector3<f32>>,
    pub normals: Option<Vec<Vector3<f32>>>,
    pub indices: Option<Vec<u32>>,
    pub textures: Option<Vec<TextureInfo>>,
    pub texture_coordinates: Option<Vec<Vector2<f32>>>,
    pub shininess: Option<f32>,
}

impl Mesh {
    pub fn flattened_data(&self) -> Vec<f32> {
        match (&self.normals, &self.texture_coordinates) {
            (None, None) => {
                self.get_flattened_vertices()
            }
            (Some(normals), None) => {
                self.flatten_with_vertices(normals)
            }
            (None, Some(texture_coordinates)) => {
                self.flatten_with_vertices(texture_coordinates)
            }
            (Some(normals), Some(texture_coordinates)) => {
                self.flatten_all_with_vertices(normals, texture_coordinates)
            }
        }
    }

    pub fn vertex_info_size(&self) -> usize {
        let normals_size = if self.normals.is_some() {
            3
        } else {
            0
        };
        let textures_size = if self.texture_coordinates.is_some() {
            2
        } else {
            0
        };
        3 + normals_size + textures_size
    }

    fn get_flattened_vertices(&self) -> Vec<f32> {
        self.vertices.iter()
            .map(|v| v.data.as_slice())
            .flatten()
            .cloned()
            .collect::<Vec<f32>>()
    }

    fn flatten_all_with_vertices(
        &self,
        normals: &[Vector3<f32>],
        texture_coordinates: &[Vector2<f32>],
    ) -> Vec<f32> {
        self.vertices.iter()
            .zip(normals)
            .zip(texture_coordinates)
            .map(|((v, n), t)| {
                let mut d = v.data.as_slice().to_vec();
                d.extend(n.data.as_slice());
                d.extend(t.data.as_slice());
                d
            })
            .flatten()
            .collect::<Vec<f32>>()
    }

    fn flatten_with_vertices<D, const S: usize>(
        &self,
        other: &[Matrix<f32, D, U1, ArrayStorage<f32, S, 1>>]
    ) -> Vec<f32> {
        self.vertices.iter()
            .zip(other)
            .map(|(v, n)| {
                let mut d = v.data.as_slice().to_vec();
                d.extend(n.data.as_slice());
                d
            })
            .flatten()
            .collect::<Vec<f32>>()
    }
}

pub struct Model(pub Vec<(Mesh, Shader)>);
pub struct Shader {
    pub(crate) vertex_array: VertexArray,
    pub(crate) vertex_buffer: Buffer,
    pub(crate) elements_buffer: Option<Buffer>,
    pub(crate) textures: Vec<Texture>
}
