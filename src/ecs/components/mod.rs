use std::sync::Arc;
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

pub struct SkipRendering;
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
pub struct InstancedMesh {
    pub offsets: Vec<Vector3<f32>>,
    pub mesh: Mesh,
}

#[derive(Clone, Debug)]
pub struct InstancedModel {
    pub offsets: Vec<Vector3<f32>>,
    pub model: Vec<(Mesh, InstancedShader)>,
}

impl InstancedModel {
    pub fn new(meshes: Vec<Mesh>, rendering: &mut RenderingSystem, offsets: Vec<Vector3<f32>>) -> Result<InstancedModel, String> {
        Ok(InstancedModel {
            offsets,
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
    pub textures: Option<Vec<TextureInfo>>,
    pub texture_coordinates: Option<Vec<Vector2<f32>>>,
    pub shininess: Option<f32>,
}

impl Mesh {
    pub fn set_program(&self, program: &Program, textures: &[Arc<Texture>]) {
        let mut diffuse_index = 0;
        let mut specular_index = 0;
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
                } else {
                    panic!("Can't happen");
                };
                program.set_uniform_i1(&format!("material.{}{}", texture_type, texture_index), info.id as i32);
            }
        }
        program.set_uniform_i1("material.n_diffuse", diffuse_index);
        program.set_uniform_i1("material.n_specular", specular_index);
        let shininess = self.shininess.clone().unwrap_or(32f32);
        program.set_uniform_f1("material.shininess", shininess);
    }

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
