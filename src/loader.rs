use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use nalgebra::{Matrix4, Vector2, Vector3};
use russimp::material::Material;
use russimp::mesh::{Mesh as AssimpMesh};
use russimp::node::Node;
use russimp::scene::{PostProcess, Scene};
use russimp::texture::{Texture, TextureType};
use russimp::Vector3D;
use crate::ecs::components::{InstancedModel, Mesh, Model, TextureInfo};
use crate::ecs::systems::rendering::RenderingSystem;

fn assimp_vector_to_algebra_vector2(input: Vector3D) -> Vector2<f32> {
    Vector2::new(input.x, input.y)
}

fn assimp_vector_to_algebra_vector(input: Vector3D) -> Vector3<f32> {
    Vector3::new(input.x, input.y, input.z)
}

fn assimp_vec_vector_to_vec_algebra_vector3(input: &Vec<Vector3D>) -> Vec<Vector3<f32>> {
    input.iter().cloned()
        .map(assimp_vector_to_algebra_vector)
        .collect()
}

fn assimp_vec_vector_to_vec_algebra_vector2(input: &Vec<Vector3D>) -> Vec<Vector2<f32>> {
    input.iter().cloned()
        .map(assimp_vector_to_algebra_vector2)
        .collect()
}

fn get_info_textures_from_texture(texture_type: &TextureType, textures: &[Texture], scene_path: &Path) -> Result<Vec<TextureInfo>, String> {
    textures.iter().map(|t| {
        Ok(TextureInfo {
            id: 0,
            texture_type: texture_type.clone(),
            path: scene_path.join(t.path.clone()).to_str().ok_or("Invalid path!")?.to_string()
        })
    }).collect::<Result<Vec<TextureInfo>, String>>()
}

fn get_textures_from_material(material: &Material, scene_path: &Path) -> Result<Vec<TextureInfo>, String> {
    Ok(material.textures.iter()
        .filter(|(t, _)| vec![TextureType::Diffuse, TextureType::Specular].contains(t))
        .map(|(texture_type, textures)| {
            get_info_textures_from_texture(texture_type, textures, scene_path)
        })
        .collect::<Result<Vec<Vec<TextureInfo>>, String>>()?
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(id, mut t)| {
            t.id = id;
            t
        })
        .collect())
}

fn assimp_mesh_to_mesh(assimp_mesh: &AssimpMesh, materials: &[Material], scene_path: &Path) -> Result<Mesh, String> {
    Ok(Mesh {
        vertices: assimp_vec_vector_to_vec_algebra_vector3(&assimp_mesh.vertices),
        normals: Some(assimp_vec_vector_to_vec_algebra_vector3(&assimp_mesh.normals)),
        indices: None,
        textures: materials.get(assimp_mesh.material_index as usize).map(|m| {
            get_textures_from_material(m, scene_path)
        }).transpose()?,
        texture_coordinates: assimp_mesh.texture_coords.get(0).clone()
            .and_then(|o| o.clone())
            .map(|v| assimp_vec_vector_to_vec_algebra_vector2(&v)),
        shininess: None
    })
}

fn extract_mesh_from_scene(scene: &Scene, mesh_id: usize, scene_path: &Path) -> Result<Mesh, String> {
    assimp_mesh_to_mesh(&scene.meshes[mesh_id], &scene.materials, scene_path)
}

fn process_node(scene: &Scene, node: Rc<RefCell<Node>>, meshes: &mut Vec<Mesh>, scene_path: &Path) -> Result<(), String> {
    meshes.extend(
        node.borrow().meshes.iter().cloned()
            .map(|mid| extract_mesh_from_scene(scene, mid as usize, scene_path))
            .collect::<Result<Vec<Mesh>, String>>()?
    );
    for child in node.borrow().children.iter() {
        process_node(scene, child.clone(), meshes, scene_path)?;
    }
    Ok(())
}

pub fn load_instanced_model(model_path: &str, rendering_system: &mut RenderingSystem, models: Vec<Matrix4<f32>>) -> Result<InstancedModel, String> {
    let meshes = load_object(model_path)?;
    InstancedModel::new(meshes, rendering_system, models)
}

pub fn load_model(model_path: &str, rendering_system: &mut RenderingSystem) -> Result<Model, String> {
    let meshes = load_object(model_path)?;
    Model::from_meshes(meshes, rendering_system)
}

fn load_object(model_path: &str) -> Result<Vec<Mesh>, String> {
    let path = Path::new(model_path)
        .parent()
        .ok_or("Invalid path!".to_string())?;
    let scene = Scene::from_file(
        model_path,
        vec![PostProcess::Triangulate, PostProcess::FlipUVs]
    ).map_err(|e| e.to_string())?;
    let root = scene.root.clone().ok_or("No root node".to_string())?;
    let mut meshes = vec![];
    process_node(&scene, root, &mut meshes, path)?;
    Ok(meshes)
}