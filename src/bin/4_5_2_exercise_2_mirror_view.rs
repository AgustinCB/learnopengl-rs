use hecs::World;
use nalgebra::{Rotation3, UnitVector3, Vector3};
use num_traits::FloatConst;
use russimp::texture::TextureType;
use learnopengl::buffer::Buffer;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{TextureInfo, Transform};
use learnopengl::ecs::systems::system::System;
use learnopengl::frame_buffer::FrameBuffer;
use learnopengl::game::Game;
use learnopengl::gl_function;
use learnopengl::light::DirectionalLight;
use learnopengl::plane::build_plane;
use learnopengl::program::Program;
use learnopengl::shader::Shader;
use learnopengl::shader_loader::ShaderType;
use learnopengl::vertex_array::VertexArray;

const MIRROR_QUAD_VERTICES: [f32; 24] = [
    -0.5f32, 0.5f32, 0f32, 1f32,
    -0.5f32, -0.5f32, 0f32, 0f32,
    0.5f32, -0.5f32, 1f32, 0f32,
    -0.5f32, 0.5f32, 0f32, 1f32,
    0.5f32, -0.5f32, 1f32, 0f32,
    0.5f32, 0.5f32, 1f32, 1f32,
];

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
    program: Program,
    mirror_vao: VertexArray,
    vao: VertexArray,
    _vbo: Buffer,
    _mirror_vbo: Buffer,
}

impl FrameBufferSystem {
    pub fn new() -> Result<FrameBufferSystem, String> {
        let frame_buffer = FrameBuffer::new(800, 600);
        let program = Program::new(vec![
            Shader::new(ShaderType::Vertex as _, include_str!("shaders/15.2-postprocessing_mirror_vertex.glsl"))?,
            Shader::new(ShaderType::Fragment as _, include_str!("shaders/15.1-postprocessing_fragment.glsl"))?
        ])?;
        program.use_program();
        program.set_uniform_i1("texture1", 0);
        let vao = VertexArray::new();
        let vbo = Buffer::new(gl::ARRAY_BUFFER);
        let mirror_vao = VertexArray::new();
        let mirror_vbo = Buffer::new(gl::ARRAY_BUFFER);
        vao.bind();
        vbo.bind();
        vbo.set_data(&QUAD_VERTICES, gl::STATIC_DRAW);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 0, 4, 2, 0, false);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 1, 4, 2, 2, false);
        mirror_vao.bind();
        mirror_vbo.bind();
        mirror_vbo.set_data(&MIRROR_QUAD_VERTICES, gl::STATIC_DRAW);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 0, 4, 2, 0, false);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 1, 4, 2, 2, false);
        Ok(FrameBufferSystem {
            frame_buffer,
            program,
            vao,
            mirror_vao,
            _vbo: vbo,
            _mirror_vbo: mirror_vbo,
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
        self.frame_buffer.bind();
        gl_function!(Enable(gl::DEPTH_TEST));
        gl_function!(ClearColor(0.1f32, 0.1f32, 0.1f32, 1f32));
        Ok(())
    }

    fn update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }

    fn late_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        FrameBuffer::unbind();
        gl_function!(ClearColor(1f32, 1f32, 1f32, 1f32));
        gl_function!(Clear(gl::COLOR_BUFFER_BIT));
        self.program.use_program();
        self.vao.bind();
        gl_function!(Disable(gl::DEPTH_TEST));
        self.frame_buffer.texture.bind(gl::TEXTURE0);
        self.program.set_uniform_matrix4("rotation", &Rotation3::identity().to_homogeneous());
        gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
        self.mirror_vao.bind();
        self.program.set_uniform_matrix4(
            "rotation",
            &Rotation3::from_euler_angles(0f32, f32::PI(), 0f32).to_homogeneous()
        );
        gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
        Ok(())
    }
}

pub fn main() -> Result<(), String> {
    let mut game = Game::new(
        "Depth testing",
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
    game.spawn_light(directional_light, &light_cube)?;
    let floor = build_plane(-0.5f32, 5f32, 2f32, vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/metal.png", env!("CARGO_MANIFEST_DIR")),
        }
    ]);
    game.spawn_mesh(&cube_mesh(vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/container.jpg", env!("CARGO_MANIFEST_DIR")),
        }
    ]), Transform {
        position: Vector3::new(-1f32, 0f32, -1f32),
        scale: Vector3::new(1f32, 1f32, 1f32),
        rotation: Rotation3::identity(),
    })?;
    game.spawn_mesh(&cube_mesh(vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/container.jpg", env!("CARGO_MANIFEST_DIR")),
        }
    ]), Transform {
        position: Vector3::new(2f32, 0f32, 0f32),
        scale: Vector3::new(1f32, 1f32, 1f32),
        rotation: Rotation3::identity(),
    })?;
    game.spawn_mesh(&floor, Transform::identity())?;
    game.play_with_fps_camera(vec![Box::new(FrameBufferSystem::new()?)])?;
    Ok(())
}