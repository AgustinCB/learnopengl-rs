use std::cell::RefCell;
use std::rc::Rc;
use hecs::World;
use nalgebra::{Rotation3, UnitVector3, Vector3};
use russimp::texture::TextureType;
use learnopengl::camera::Camera;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{SkipRendering, Shader as RenderingShader, Skybox, TextureInfo, Transform, Mesh, Border, Transparent};
use learnopengl::ecs::systems::system::System;
use learnopengl::game::Game;
use learnopengl::gl_function;
use learnopengl::light::DirectionalLight;
use learnopengl::program::Program;
use learnopengl::shader::Shader;
use learnopengl::shader_loader::ShaderType;
use learnopengl::vertex_array::VertexArray;

struct Refract;

struct RefractionSystem {
    camera: Rc<RefCell<Camera>>,
    program: Program
}

impl RefractionSystem {
    fn new(camera: Rc<RefCell<Camera>>) -> Result<RefractionSystem, String> {
        Ok(RefractionSystem {
            camera,
            program: Program::new(vec![
                Shader::new(ShaderType::Vertex as _, include_str!("shaders/16.2-reflective_vertex.glsl"))?,
                Shader::new(ShaderType::Fragment as _, include_str!("shaders/16.3-refraction_fragment.glsl"))?,
            ])?,
        })
    }
}

impl System for RefractionSystem {
    fn name(&self) -> &str {
        "Refraction System"
    }

    fn start(&self, _world: &mut World) -> Result<(), String> {
        self.program.use_program();
        self.program.set_uniform_i1("skybox", 0);
        Ok(())
    }

    fn early_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }

    fn update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }

    fn late_update(&self, world: &mut World, _delta_time: f32) -> Result<(), String> {
        let skybox = world.query_mut::<&RenderingShader>().with::<Skybox>().into_iter().next()
            .map(|(_e, s) | s.textures.get(0).unwrap().clone());
        if let Some(texture) = skybox {
            texture.bind(gl::TEXTURE0);
            self.program.use_program();
            let view = self.camera.borrow().look_at_matrix();
            let projection = self.camera.borrow().projection();
            self.program.set_uniform_matrix4("view", &view);
            self.program.set_uniform_v3("viewPos", self.camera.borrow().position());
            self.program.set_uniform_matrix4("projection", &projection);
            for (_e, (_mesh, shader, transform)) in world.query::<(&Mesh, &RenderingShader, &Transform)>().without::<Border>().without::<Transparent>().with::<SkipRendering>().iter() {
                self.program.set_uniform_matrix4("model", &transform.get_model_matrix());
                shader.vertex_array.bind();
                gl_function!(DrawArrays(gl::TRIANGLES, 0, 36));
                VertexArray::unbind();
            }
        }
        Ok(())
    }
}

pub fn main() -> Result<(), String> {
    let mut game = Game::new(
        "Refraction",
        800,
        600,
        120,
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
    let skybox = game.spawn_skybox(&Skybox {
        texture_info: TextureInfo {
            id: 0,
            texture_type: TextureType::None,
            path: format!("{}/resource/skybox", env!("CARGO_MANIFEST_DIR")),
        }
    })?;
    game.add_to(skybox, directional_light)?;
    let e = game.spawn_mesh(&cube_mesh(vec![]), Transform {
        position: Vector3::new(0f32, 0f32, 0f32),
        scale: Vector3::new(1f32, 1f32, 1f32),
        rotation: Rotation3::identity(),
    })?;
    game.add_to(e, SkipRendering)?;
    game.add_to(e, Refract)?;
    game.play_with_fps_camera(vec![
        Box::new(RefractionSystem::new(game.camera())?),
    ])?;
    Ok(())
}