use std::sync::Arc;
use itertools::multizip;
use nalgebra::{ArrayStorage, Matrix, Matrix4, Rotation3, Scale3, Translation3, U1, Vector2, Vector3};
use russimp::texture::TextureType;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use crate::buffer::Buffer;
use crate::ecs::systems::input::InputType;
use crate::ecs::systems::rendering::RenderingSystem;
use crate::program::Program;
use crate::texture::Texture;
use crate::vertex_array::VertexArray;

pub fn get_flattened_vectors(vectors: &[Vector3<f32>]) -> Vec<f32> {
    vectors.iter()
        .map(|v| v.data.as_slice())
        .flatten()
        .cloned()
        .collect::<Vec<f32>>()
}

pub fn get_flattened_matrices(matrices: &[Matrix4<f32>]) -> Vec<f32> {
    matrices.iter()
        .map(|m| m.as_slice())
        .flatten()
        .cloned()
        .collect()
}

#[derive(Clone, Debug)]
pub enum UniformValue {
    Float(f32),
    Texture(u32),
    Matrix(Matrix4<f32>),
    Vector3(Vector3<f32>),
}
#[derive(Clone, Debug)]
pub struct ExtraUniform {
    pub name: &'static str,
    pub value: UniformValue,
}
#[derive(Clone, Debug)]
pub struct SkipRendering;
#[derive(Clone, Debug)]
pub struct Transparent;

#[derive(Clone, Debug)]
pub struct Input {
    pub input_types: Vec<InputType>,
    pub events: Vec<Event>,
}

impl Input {
    pub fn new(input_types: Vec<InputType>) -> Input {
        Input {
            input_types,
            events: vec![],
        }
    }
}

#[derive(Clone, Debug)]
pub struct FpsCamera {
    pub camera_speed: f32,
}

#[derive(Clone, Debug)]
pub struct QuitControl {
    pub quit_keycode: Keycode,
}

#[derive(Clone, Debug)]
pub struct TextureInfo {
    pub id: usize,
    pub texture_type: TextureType,
    pub path: String,
}

#[derive(Clone, Debug)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Rotation3<f32>,
    pub scale: Vector3<f32>,
}

impl Transform {
    pub fn identity() -> Transform {
        Transform {
            position: Vector3::new(0f32, 0f32, 0f32),
            rotation: Rotation3::identity(),
            scale: Vector3::new(1f32, 1f32, 1f32),
        }
    }

    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        let t = Translation3::from(self.position);
        let s = Scale3::from(self.scale);
        t.to_homogeneous() * self.rotation.to_homogeneous() * s.to_homogeneous()
    }
}

#[derive(Clone, Debug)]
pub struct Velocity(pub Vector3<f32>);

#[derive(Clone, Debug)]
pub struct Border {
    pub color: Vector3<f32>,
    pub scale: f32,
}

pub(crate) const SKYBOX_VERTICES: [f32; 108] = [
    -1.0f32,  1.0f32, -1.0f32,
    -1.0f32, -1.0f32, -1.0f32,
    1.0f32, -1.0f32, -1.0f32,
    1.0f32, -1.0f32, -1.0f32,
    1.0f32,  1.0f32, -1.0f32,
    -1.0f32,  1.0f32, -1.0f32,
    -1.0f32, -1.0f32,  1.0f32,
    -1.0f32, -1.0f32, -1.0f32,
    -1.0f32,  1.0f32, -1.0f32,
    -1.0f32,  1.0f32, -1.0f32,
    -1.0f32,  1.0f32,  1.0f32,
    -1.0f32, -1.0f32,  1.0f32,
    1.0f32, -1.0f32, -1.0f32,
    1.0f32, -1.0f32,  1.0f32,
    1.0f32,  1.0f32,  1.0f32,
    1.0f32,  1.0f32,  1.0f32,
    1.0f32,  1.0f32, -1.0f32,
    1.0f32, -1.0f32, -1.0f32,
    -1.0f32, -1.0f32,  1.0f32,
    -1.0f32,  1.0f32,  1.0f32,
    1.0f32,  1.0f32,  1.0f32,
    1.0f32,  1.0f32,  1.0f32,
    1.0f32, -1.0f32,  1.0f32,
    -1.0f32, -1.0f32,  1.0f32,
    -1.0f32,  1.0f32, -1.0f32,
    1.0f32,  1.0f32, -1.0f32,
    1.0f32,  1.0f32,  1.0f32,
    1.0f32,  1.0f32,  1.0f32,
    -1.0f32,  1.0f32,  1.0f32,
    -1.0f32,  1.0f32, -1.0f32,
    -1.0f32, -1.0f32, -1.0f32,
    -1.0f32, -1.0f32,  1.0f32,
    1.0f32, -1.0f32, -1.0f32,
    1.0f32, -1.0f32, -1.0f32,
    -1.0f32, -1.0f32,  1.0f32,
    1.0f32, -1.0f32,  1.0f32,
];

#[derive(Clone, Debug)]
pub struct Skybox {
    pub texture_info: TextureInfo,
}

#[derive(Clone, Debug)]
pub struct WithNormals;

#[derive(Clone, Debug)]
pub struct InstancedMesh {
    pub models: Vec<Matrix4<f32>>,
    pub mesh: Mesh,
}

#[derive(Clone, Debug)]
pub struct InstancedModel {
    pub models: Vec<Matrix4<f32>>,
    pub model: Vec<(Mesh, InstancedShader)>,
}

impl InstancedModel {
    pub fn new(meshes: Vec<Mesh>, rendering: &mut RenderingSystem, models: Vec<Matrix4<f32>>) -> Result<InstancedModel, String> {
        Ok(InstancedModel {
            models,
            model: meshes.into_iter().map(|m| {
                let shader = rendering.shader_for_mesh(&m)?;
                let shader = rendering.instanced_rendering.shader_for_mesh(&shader)?;
                Ok((m, shader))
            }).collect::<Result<Vec<_>, String>>()?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vector3<f32>>,
    pub normals: Option<Vec<Vector3<f32>>>,
    pub indices: Option<Vec<u32>>,
    pub tangents: Option<Vec<Vector3<f32>>>,
    pub bitangents: Option<Vec<Vector3<f32>>>,
    pub textures: Option<Vec<TextureInfo>>,
    pub texture_coordinates: Option<Vec<Vector2<f32>>>,
    pub shininess: Option<f32>,
}

impl Mesh {
    pub fn len(&self) -> usize {
        if let Some(indices) = &self.indices {
            indices.len()
        } else {
            self.vertices.len()
        }
    }

    pub fn set_program(&self, program: &Program, textures: &[Arc<Texture>]) {
        let mut diffuse_index = 0;
        let mut specular_index = 0;
        let mut normal_index = 0;
        let mut height_index = 0;
        if let Some(infos) = &self.textures {
            for (texture, info) in textures.iter().zip(infos.iter()) {
                texture.bind(gl::TEXTURE0 + info.id as u32);
                let (texture_type, texture_index) = if info.texture_type == TextureType::Diffuse {
                    let index = diffuse_index;
                    diffuse_index += 1;
                    ("diffuse", index)
                } else if info.texture_type == TextureType::Specular {
                    let index = specular_index;
                    specular_index += 1;
                    ("specular", index)
                } else if info.texture_type == TextureType::Normals {
                    let index = normal_index;
                    normal_index += 1;
                    ("normal", index)
                } else if info.texture_type == TextureType::Height {
                    let index = height_index;
                    height_index += 1;
                    ("height", index)
                } else {
                    panic!("Can't happen");
                };
                program.set_uniform_i1(&format!("material.{}{}", texture_type, texture_index), info.id as i32);
            }
        }
        program.set_uniform_i1("material.n_diffuse", diffuse_index);
        program.set_uniform_i1("material.n_specular", specular_index);
        program.set_uniform_i1("material.n_height", height_index);
        let shininess = self.shininess.clone().unwrap_or(64f32);
        program.set_uniform_f1("material.shininess", shininess);
    }

    pub fn flattened_data(&self) -> Vec<f32> {
        match (&self.normals, &self.texture_coordinates, &self.tangents, &self.bitangents) {
            (None, None, None, None) => {
                self.get_flattened_vertices()
            }
            (Some(normals), None, None, None) => {
                self.flatten_with_vertices(normals)
            }
            (None, Some(texture_coordinates), None, None) => {
                self.flatten_with_vertices(texture_coordinates)
            }
            (None, None, Some(tangents), None) => {
                self.flatten_with_vertices(tangents)
            }
            (None, None, None, Some(bitangents)) => {
                self.flatten_with_vertices(bitangents)
            }
            (Some(normals), Some(texture_coordinates), None, None) => {
                self.flatten_two_with_vertices(normals, texture_coordinates)
            }
            (Some(normals), None, Some(tangents), None) => {
                self.flatten_two_with_vertices(normals, tangents)
            }
            (Some(normals), None, None, Some(bitangents)) => {
                self.flatten_two_with_vertices(normals, bitangents)
            }
            (None, Some(texture_coordinates), Some(tangents), None) => {
                self.flatten_two_with_vertices(texture_coordinates, tangents)
            }
            (None, Some(texture_coordinates), None, Some(bitangents)) => {
                self.flatten_two_with_vertices(texture_coordinates, bitangents)
            }
            (None, None, Some(tangents), Some(bitangents)) => {
                self.flatten_two_with_vertices(tangents, bitangents)
            }
            (Some(normals), Some(texture_coordinates), Some(tangents), None) => {
                self.flatten_three_with_vertices(normals, texture_coordinates, tangents)
            }
            (Some(normals), Some(texture_coordinates), None, Some(bitangents)) => {
                self.flatten_three_with_vertices(normals, texture_coordinates, bitangents)
            }
            (Some(normals), None, Some(tangents), Some(bitangents)) => {
                self.flatten_three_with_vertices(normals, tangents, bitangents)
            }
            (None, Some(texture_coordinates), Some(tangents), Some(bitangents)) => {
                self.flatten_three_with_vertices(texture_coordinates, tangents, bitangents)
            }
            (Some(normals), Some(texture_coordinates), Some(tangents), Some(bitangents)) => {
                self.flatten_four_with_vertices(normals, texture_coordinates, tangents, bitangents)
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
        let tangents_size = if self.tangents.is_some() {
            3
        } else {
            0
        };
        let bitangents_size = if self.bitangents.is_some() {
            3
        } else {
            0
        };
        3 + normals_size + textures_size + tangents_size + bitangents_size
    }

    fn get_flattened_vertices(&self) -> Vec<f32> {
        get_flattened_vectors(&self.vertices)
    }

    fn flatten_two_with_vertices<U, V, const C: usize, const C1: usize>(
        &self,
        first: &[Matrix<f32, U, U1, ArrayStorage<f32, C, 1>>],
        second: &[Matrix<f32, V, U1, ArrayStorage<f32, C1, 1>>],
    ) -> Vec<f32> {
        multizip((self.vertices.iter(), first, second))
            .map(|(v, n, t)| {
                let mut d = v.data.as_slice().to_vec();
                d.extend(n.data.as_slice());
                d.extend(t.data.as_slice());
                d
            })
            .flatten()
            .collect::<Vec<f32>>()
    }

    fn flatten_three_with_vertices<U, V, W, const C: usize, const C1: usize, const C2: usize>(
        &self,
        first: &[Matrix<f32, U, U1, ArrayStorage<f32, C, 1>>],
        second: &[Matrix<f32, V, U1, ArrayStorage<f32, C1, 1>>],
        third: &[Matrix<f32, W, U1, ArrayStorage<f32, C2, 1>>],
    ) -> Vec<f32> {
        multizip((self.vertices.iter(), first, second, third))
            .map(|(v, f, s, t)| {
                let mut d = v.data.as_slice().to_vec();
                d.extend(f.data.as_slice());
                d.extend(s.data.as_slice());
                d.extend(t.data.as_slice());
                d
            })
            .flatten()
            .collect::<Vec<f32>>()
    }

    fn flatten_four_with_vertices<U, V, W, Y, const C: usize, const C1: usize, const C2: usize, const C3: usize>(
        &self,
        first: &[Matrix<f32, U, U1, ArrayStorage<f32, C, 1>>],
        second: &[Matrix<f32, V, U1, ArrayStorage<f32, C1, 1>>],
        third: &[Matrix<f32, W, U1, ArrayStorage<f32, C2, 1>>],
        forth: &[Matrix<f32, Y, U1, ArrayStorage<f32, C3, 1>>],
    ) -> Vec<f32> {
        multizip((self.vertices.iter(), first, second, third, forth))
            .map(|(v, f, s, t, ff)| {
                let mut d = v.data.as_slice().to_vec();
                d.extend(f.data.as_slice());
                d.extend(s.data.as_slice());
                d.extend(t.data.as_slice());
                d.extend(ff.data.as_slice());
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

#[derive(Clone, Debug)]
pub struct Model(pub Vec<(Mesh, Shader)>);

impl Model {
    pub fn from_meshes(meshes: Vec<Mesh>, rendering: &mut RenderingSystem) -> Result<Model, String> {
        Ok(Model(
            meshes.into_iter()
                .map(|m| {
                    let shader = rendering.shader_for_mesh(&m)?;
                    Ok((m, shader))
                })
                .collect::<Result<Vec<(Mesh, Shader)>, String>>()?
        ))
    }
}

#[derive(Debug, Clone)]
pub struct InstancedShader {
    pub vertex_array: Arc<VertexArray>,
    pub(crate) vertex_buffer: Arc<Buffer>,
    pub(crate) offset_buffer: Arc<Buffer>,
    pub(crate) elements_buffer: Option<Arc<Buffer>>,
    pub textures: Vec<Arc<Texture>>
}

#[derive(Debug, Clone)]
pub struct Shader {
    pub vertex_array: Arc<VertexArray>,
    pub(crate) vertex_buffer: Arc<Buffer>,
    pub(crate) elements_buffer: Option<Arc<Buffer>>,
    pub textures: Vec<Arc<Texture>>
}
