use hecs::World;
use nalgebra::{Rotation3, UnitVector3, Vector3};
use russimp::texture::TextureType;
use learnopengl::buffer::Buffer;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{Skybox, TextureInfo, Transform};
use learnopengl::ecs::systems::system::System;
use learnopengl::frame_buffer::FrameBuffer;
use learnopengl::game::Game;
use learnopengl::gl_function;
use learnopengl::light::DirectionalLight;
use learnopengl::program::Program;
use learnopengl::shader::Shader;
use learnopengl::shader_loader::ShaderType;
use learnopengl::vertex_array::VertexArray;

const QUAD_VERTICES: [f32; 24] = [
    -1f32, 1f32, 0f32, 1f32,
    -1f32, -1f32, 0f32, 0f32,
    1f32, -1f32, 1f32, 0f32,
    -1f32, 1f32, 0f32, 1f32,
    1f32, -1f32, 1f32, 0f32,
    1f32, 1f32, 1f32, 1f32,
];

struct FrameBufferSystem {
    frame_buffer: FrameBuffer,
    intermediate_frame_buffer: FrameBuffer,
    program: Program,
    vao: VertexArray,
    _vbo: Buffer,
}

impl FrameBufferSystem {
    pub fn new() -> Result<FrameBufferSystem, String> {
        let frame_buffer = FrameBuffer::multisample(800, 600);
        let intermediate_frame_buffer = FrameBuffer::intermediate(800, 600);
        let program = Program::new(vec![
            Shader::new(ShaderType::Vertex as _, include_str!("shaders/15.1-postprocessing_vertex.glsl"))?,
            Shader::new(ShaderType::Fragment as _, include_str!("shaders/15.1-postprocessing_fragment.glsl"))?
        ])?;
        program.use_program();
        program.set_uniform_i1("texture1", 0);
        let vao = VertexArray::new();
        let vbo = Buffer::new(gl::ARRAY_BUFFER);
        vao.bind();
        vbo.bind();
        vbo.set_data(&QUAD_VERTICES, gl::STATIC_DRAW);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 0, 4, 2, 0, false);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 1, 4, 2, 2, false);
        Ok(FrameBufferSystem {
            frame_buffer,
            intermediate_frame_buffer,
            program,
            vao,
            _vbo: vbo,
        })
    }
}

impl System for FrameBufferSystem {
    fn name(&self) -> &str {
        "Frame Buffer"
    }

    fn start(&self, _world: &mut World) -> Result<(), String> {
        Ok(())
    }

    fn early_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        gl_function!(ClearColor(0.1f32, 0.1f32, 0.1f32, 1f32));
        gl_function!(Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
        self.frame_buffer.bind();
        gl_function!(ClearColor(0.1f32, 0.1f32, 0.1f32, 1f32));
        gl_function!(Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
        gl_function!(Enable(gl::DEPTH_TEST));
        Ok(())
    }

    fn update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }

    fn late_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        self.frame_buffer.read_bind();
        self.intermediate_frame_buffer.draw_bind();
        gl_function!(BlitFramebuffer(0, 0, 800, 600, 0, 0, 800, 600, gl::COLOR_BUFFER_BIT, gl::NEAREST));

        FrameBuffer::unbind();
        gl_function!(ClearColor(1f32, 1f32, 1f32, 1f32));
        gl_function!(Clear(gl::COLOR_BUFFER_BIT));
        gl_function!(Disable(gl::DEPTH_TEST));

        self.program.use_program();
        self.vao.bind();
        self.intermediate_frame_buffer.texture.bind(gl::TEXTURE0);
        gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
        Ok(())
    }
}

pub fn main() -> Result<(), String> {
    let mut game = Game::new_with_anti_alias(
        "Anti Aliasing Offscreen",
        800,
        600,
        120,
        Vector3::new(0f32, 0f32, 0f32),
        "17.1-uniform_buffer_objects_vertex.glsl",
        "12.1-modelloading.glsl",
        "17.1-uniform_buffer_objects_vertex.glsl",
        "09.1-lightfragment.glsl",
        4
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
    game.spawn_mesh(&cube_mesh(vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/container.jpg", env!("CARGO_MANIFEST_DIR")),
        }
    ]), Transform {
        position: Vector3::new(0f32, 0f32, 0f32),
        scale: Vector3::new(1f32, 1f32, 1f32),
        rotation: Rotation3::identity(),
    })?;
    game.play_with_fps_camera(vec![Box::new(FrameBufferSystem::new()?)])?;
    Ok(())
}