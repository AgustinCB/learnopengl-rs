use std::cell::RefCell;
use std::rc::Rc;
use hecs::World;
use include_dir::{Dir, include_dir};
use nalgebra::{Rotation3, UnitVector3, Vector3};
use russimp::texture::TextureType;
use learnopengl::camera::Camera;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{Model, SkipRendering, Transform};
use learnopengl::ecs::systems::system::System;
use learnopengl::game::Game;
use learnopengl::gl_function;
use learnopengl::light::{DirectionalLight, PointLight};
use learnopengl::program::Program;
use learnopengl::shader_loader::{ShaderLoader, ShaderType};
use learnopengl::vertex_array::VertexArray;

static SHADERS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/src/bin/shaders");

struct ExplodingSystem {
    camera: Rc<RefCell<Camera>>,
    program: Program,
}

impl ExplodingSystem {
    fn new(camera: Rc<RefCell<Camera>>) -> Result<ExplodingSystem, String> {
        let shader_loader = ShaderLoader::new(&SHADERS_DIR);
        let program = Program::new(vec![
            shader_loader.load(ShaderType::Vertex, "18.1-exploding_geometry_vertex.glsl")?,
            shader_loader.load(ShaderType::Geometry, "18.1-exploding_geometry.glsl")?,
            shader_loader.load(ShaderType::Fragment, "18.1-exploding_geometry_fragment.glsl")?,
        ])?;
        Ok(ExplodingSystem {
            camera,
            program
        })
    }
}

impl System for ExplodingSystem {
    fn name(&self) -> &str {
        "Exploding system"
    }

    fn start(&self, _world: &mut World) -> Result<(), String> {
        Ok(())
    }

    fn early_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }

    fn update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }

    fn late_update(&self, world: &mut World, delta_time: f32) -> Result<(), String> {
        let projection = (*self.camera).borrow().projection();
        let view = (*self.camera).borrow().look_at_matrix();
        self.program.use_program();
        self.program.set_uniform_matrix4("projection", &projection);
        self.program.set_uniform_matrix4("view", &view);
        self.program.set_uniform_f1("time", delta_time);
        for (_e, (model, transform)) in world.query_mut::<(&Model, &Transform)>().with::<SkipRendering>() {
            self.program.set_uniform_matrix4("model", &transform.get_model_matrix());
            gl_function!(StencilMask(0x00));
            for (mesh, shader) in model.0.iter() {
                let mut diffuse_index = 0;
                if let Some(infos) = &mesh.textures {
                    for (texture, info) in shader.textures.iter().zip(infos.iter()) {
                        texture.bind(gl::TEXTURE0 + info.id as u32);
                        if info.texture_type == TextureType::Diffuse {
                            diffuse_index += 1;
                            self.program.set_uniform_i1(&format!("texture_diffuse{}", diffuse_index), info.id as i32);
                        };
                    }
                }
                let n_vertices = mesh.vertices.len();
                shader.vertex_array.bind();
                gl_function!(DrawArrays(gl::TRIANGLES, 0, n_vertices as i32));
                VertexArray::unbind();
            }
        }
        Ok(())
    }
}

pub fn main() -> Result<(), String> {
    let mut game = Game::new(
        "Model loading",
        800,
        600,
        60,
        Vector3::new(0f32, 0f32, 0f32),
        "17.1-uniform_buffer_objects_vertex.glsl",
        "12.1-modelloading.glsl",
        "17.1-uniform_buffer_objects_vertex.glsl",
        "09.1-lightfragment.glsl",
    )?;
    let directional_light = DirectionalLight::new(
        UnitVector3::new_normalize(Vector3::new(-0.2f32, -1f32, -0.3f32)),
        Vector3::new(0.2f32, 0.2f32, 0.2f32),
        Vector3::new(0.5f32, 0.5f32, 0.5f32),
        Vector3::new(1f32, 1f32, 1f32),
    );
    let light_cube = cube_mesh(vec![]);
    game.spawn_light(directional_light.clone(), &light_cube)?;
    let point_light = PointLight::new(
        Vector3::new(0f32,  4f32,  0f32),
        Vector3::new(0.2f32, 0.2f32, 0.2f32),
        Vector3::new(0.5f32, 0.5f32, 0.5f32),
        Vector3::new(1f32, 1f32, 1f32),
        1f32,
        0.09f32,
        0.032f32,
    );
    game.spawn_light(point_light, &light_cube)?;
    let model_path = format!("{}/../LOGL/resources/objects/backpack/backpack.obj", env!("CARGO_MANIFEST_DIR"));
    let e = game.spawn_model_from_file(&model_path, Transform {
        position: Vector3::new(0f32, 0f32, 0f32),
        rotation: Rotation3::identity(),
        scale: Vector3::new(1f32, 1f32, 1f32),
    })?;
    game.add_to(e, SkipRendering)?;
    game.play_with_fps_camera(vec![
        Box::new(ExplodingSystem::new(game.camera())?)
    ])?;
    Ok(())
}