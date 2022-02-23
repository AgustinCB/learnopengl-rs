use std::cell::RefCell;
use hecs::World;
use nalgebra::{Rotation3, Vector3};
use russimp::texture::TextureType;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use learnopengl::buffer::Buffer;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{Input, TextureInfo, Transform};
use learnopengl::ecs::systems::input::InputType;
use learnopengl::ecs::systems::system::System;
use learnopengl::frame_buffer::FrameBuffer;
use learnopengl::game::Game;
use learnopengl::gl_function;
use learnopengl::light::PointLight;
use learnopengl::program::Program;
use learnopengl::shader::Shader;
use learnopengl::shader_loader::ShaderType;
use learnopengl::texture::TextureFormat;
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
    exposure: RefCell<f32>,
    frame_buffer: FrameBuffer,
    program: Program,
    vao: VertexArray,
    _vbo: Buffer,
}

struct ExposureControl(Keycode, Keycode);

impl FrameBufferSystem {
    pub fn new() -> Result<FrameBufferSystem, String> {
        let frame_buffer = FrameBuffer::new_with_format(800, 600, TextureFormat::FloatingPoint);
        let program = Program::new(vec![
            Shader::new(ShaderType::Vertex as _, include_str!("shaders/15.1-postprocessing_vertex.glsl"))?,
            Shader::new(ShaderType::Fragment as _, include_str!("shaders/24.1-hdr_fragment.glsl"))?
        ])?;
        program.use_program();
        program.set_uniform_i1("texture1", 0);
        program.set_uniform_f1("exposure", 1f32);
        let vao = VertexArray::new();
        let vbo = Buffer::new(gl::ARRAY_BUFFER);
        vao.bind();
        vbo.bind();
        vbo.set_data(&QUAD_VERTICES, gl::STATIC_DRAW);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 0, 4, 2, 0, false);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 1, 4, 2, 2, false);
        Ok(FrameBufferSystem {
            exposure: RefCell::new(1f32),
            frame_buffer,
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
        self.frame_buffer.bind();
        gl_function!(Enable(gl::DEPTH_TEST));
        gl_function!(ClearColor(0.1f32, 0.1f32, 0.1f32, 1f32));
        Ok(())
    }

    fn update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        for (_e, (input, exposure_control)) in _world.query_mut::<(&Input, &ExposureControl)>() {
            for e in input.events.iter() {
                let mut exposure = *self.exposure.borrow();
                match &e {
                    Event::KeyDown { keycode: Some(k), .. } if k == &exposure_control.0 => {
                        exposure -= 1f32;
                        self.program.set_uniform_f1("exposure", exposure);
                        self.exposure.replace(exposure);
                    }
                    Event::KeyDown { keycode: Some(k), .. } if k == &exposure_control.1 => {
                        exposure += 1f32;
                        self.program.set_uniform_f1("exposure", exposure);
                        self.exposure.replace(exposure);
                    }
                    _ => {},
                }
            }
        }
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
        gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
        Ok(())
    }
}

pub fn main() -> Result<(), String> {
    let mut game = Game::new(
        "HDR",
        800,
        600,
        60,
        Vector3::new(0f32, 0f32, 0f32),
        "17.1-uniform_buffer_objects_vertex.glsl",
        "12.1-modelloading.glsl",
        "17.1-uniform_buffer_objects_vertex.glsl",
        "09.1-lightfragment.glsl",
    )?;
    let mut cube = cube_mesh(vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/wood.png", env!("CARGO_MANIFEST_DIR")),
        }
    ]);
    for (position, color) in vec![
        (Vector3::new(0f32, 0f32, 49.5f32), Vector3::new(200f32, 200f32, 200f32)),
        (Vector3::new(-1.4f32, -1.9f32, 9f32), Vector3::new(0.1f32, 0f32, 0f32)),
        (Vector3::new(0f32, -1.8f32, 4f32), Vector3::new(0f32, 0f32, 0.2f32)),
        (Vector3::new(0f32, -1.7f32, 6f32), Vector3::new(0f32, 0.1f32, 0f32)),
    ] {
        let point_light = PointLight::new(
            position,
            Vector3::zeros(),
            color,
            Vector3::zeros(),
            0f32,
            0f32,
            1f32,
        );
        let l = game.spawn(());
        game.add_to(l, point_light)?;
    }
    cube.normals = Some(cube.normals.clone().unwrap().into_iter()
        .map(|v| (-v).normalize())
        .collect());
    game.spawn_mesh(&cube, Transform {
        position: Vector3::new(0f32, 0f32, 25f32),
        scale: Vector3::new(5f32, 5f32, 55f32),
        rotation: Rotation3::identity(),
    })?;
    game.spawn((Input::new(vec![InputType::Keyboard]), ExposureControl(Keycode::Q, Keycode::E)));
    game.play_with_fps_camera(vec![Box::new(FrameBufferSystem::new()?)])?;
    Ok(())
}