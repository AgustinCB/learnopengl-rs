use std::cell::RefCell;
use hecs::World;
use nalgebra::{Rotation3, UnitVector3, Vector3};
use russimp::texture::TextureType;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use learnopengl::buffer::Buffer;
use learnopengl::cube::cube_mesh;
use learnopengl::ecs::components::{Input, TextureInfo, Transform};
use learnopengl::ecs::systems::input::InputType;
use learnopengl::ecs::systems::system::System;
use learnopengl::game::Game;
use learnopengl::gl_function;
use learnopengl::light::PointLight;
use learnopengl::multiple_render_target::MultipleRenderTarget;
use learnopengl::ping_pong_frame_buffer::PingPongFrameBuffer;
use learnopengl::plane::build_plane;
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

#[derive(Clone, Copy)]
enum RenderingTarget {
    Hdr,
    Brightness,
    Blur,
    Bloom,
}

struct FrameBufferSystem {
    bloom_program: Program,
    blur_frame_buffer: PingPongFrameBuffer,
    blur_program: Program,
    exposure: RefCell<f32>,
    frame_buffer: MultipleRenderTarget,
    program: Program,
    target: RefCell<RenderingTarget>,
    vao: VertexArray,
    _vbo: Buffer,
}

struct BloomControl{
    bloom: Keycode,
    blur: Keycode,
    brightness: Keycode,
    increase_exposure: Keycode,
    decrease_exposure: Keycode,
    hdr: Keycode,
}

impl FrameBufferSystem {
    pub fn new() -> Result<FrameBufferSystem, String> {
        let frame_buffer = MultipleRenderTarget::new_with_format(800, 600, 2, TextureFormat::FloatingPoint);
        let blur_frame_buffer = PingPongFrameBuffer::new_with_format(800, 600, TextureFormat::FloatingPoint);
        let program = Program::new(vec![
            Shader::new(ShaderType::Vertex as _, include_str!("shaders/15.1-postprocessing_vertex.glsl"))?,
            Shader::new(ShaderType::Fragment as _, include_str!("shaders/24.1-hdr_fragment.glsl"))?
        ])?;
        let bloom_program = Program::new(vec![
            Shader::new(ShaderType::Vertex as _, include_str!("shaders/25.1-blur_vertex.glsl"))?,
            Shader::new(ShaderType::Fragment as _, include_str!("shaders/25.1-bloom_final_fragment.glsl"))?,
        ])?;
        let blur_program = Program::new(vec![
            Shader::new(ShaderType::Vertex as _, include_str!("shaders/25.1-blur_vertex.glsl"))?,
            Shader::new(ShaderType::Fragment as _, include_str!("shaders/25.1-blur_fragment.glsl"))?
        ])?;
        program.use_program();
        program.set_uniform_i1("texture1", 0);
        program.set_uniform_f1("exposure", 1f32);
        bloom_program.use_program();
        bloom_program.set_uniform_i1("scene", 0);
        bloom_program.set_uniform_i1("bloomBlur", 1);
        bloom_program.set_uniform_f1("exposure", 1f32);
        blur_program.use_program();
        blur_program.set_uniform_i1("image", 0);
        let vao = VertexArray::new();
        let vbo = Buffer::new(gl::ARRAY_BUFFER);
        vao.bind();
        vbo.bind();
        vbo.set_data(&QUAD_VERTICES, gl::STATIC_DRAW);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 0, 4, 2, 0, false);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 1, 4, 2, 2, false);
        Ok(FrameBufferSystem {
            bloom_program,
            blur_frame_buffer,
            blur_program,
            frame_buffer,
            program,
            vao,
            exposure: RefCell::new(1f32),
            target: RefCell::new(RenderingTarget::Bloom),
            _vbo: vbo,
        })
    }
}

impl System for FrameBufferSystem {
    fn name(&self) -> &str {
        "Frame Buffer"
    }

    fn start(&self, _world: &mut World) -> Result<(), String> {
        self.frame_buffer.bind();
        self.frame_buffer.set_draw_buffers();
        MultipleRenderTarget::unbind();
        Ok(())
    }

    fn early_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        self.frame_buffer.bind();
        gl_function!(Enable(gl::DEPTH_TEST));
        gl_function!(ClearColor(0f32, 0f32, 0f32, 1f32));
        Ok(())
    }

    fn update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        for (_e, (input, exposure_control)) in _world.query_mut::<(&Input, &BloomControl)>() {
            for e in input.events.iter() {
                let mut exposure = *self.exposure.borrow();
                match &e {
                    Event::KeyDown { keycode: Some(k), .. } if k == &exposure_control.decrease_exposure => {
                        exposure -= 1f32;
                        self.program.use_program();
                        self.program.set_uniform_f1("exposure", exposure);
                        self.blur_program.use_program();
                        self.blur_program.set_uniform_f1("exposure", exposure);
                        self.exposure.replace(exposure);
                    }
                    Event::KeyDown { keycode: Some(k), .. } if k == &exposure_control.increase_exposure => {
                        exposure += 1f32;
                        self.program.use_program();
                        self.program.set_uniform_f1("exposure", exposure);
                        self.blur_program.use_program();
                        self.blur_program.set_uniform_f1("exposure", exposure);
                        self.exposure.replace(exposure);
                    }
                    Event::KeyDown { keycode: Some(k), .. } if k == &exposure_control.hdr => {
                        self.target.replace(RenderingTarget::Hdr);
                    }
                    Event::KeyDown { keycode: Some(k), .. } if k == &exposure_control.brightness => {
                        self.target.replace(RenderingTarget::Brightness);
                    }
                    Event::KeyDown { keycode: Some(k), .. } if k == &exposure_control.blur => {
                        self.target.replace(RenderingTarget::Blur);
                    }
                    Event::KeyDown { keycode: Some(k), .. } if k == &exposure_control.bloom => {
                        self.target.replace(RenderingTarget::Bloom);
                    }
                    _ => {},
                }
            }
        }
        Ok(())
    }

    fn late_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        MultipleRenderTarget::unbind();
        self.vao.bind();
        gl_function!(Disable(gl::DEPTH_TEST));

        self.blur_program.use_program();
        let mut horizontal = true;
        let mut first_iteration = true;
        for _ in 0..10 {
            self.blur_frame_buffer.bind(horizontal, 0);
            if first_iteration {
                self.frame_buffer.textures.get(1).unwrap().bind(gl::TEXTURE0);
            }
            self.blur_program.set_uniform_i1("horizontal", horizontal as i32);
            gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
            horizontal = !horizontal;
            if first_iteration {
                first_iteration = false;
            }
        }
        PingPongFrameBuffer::unbind();

        gl_function!(ClearColor(1f32, 1f32, 1f32, 1f32));
        gl_function!(Clear(gl::COLOR_BUFFER_BIT));
        match *self.target.borrow() {
            RenderingTarget::Hdr => {
                self.program.use_program();
                self.frame_buffer.textures.get(0).unwrap().bind(gl::TEXTURE0);
                gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
            },
            RenderingTarget::Brightness => {
                self.program.use_program();
                self.frame_buffer.textures.get(1).unwrap().bind(gl::TEXTURE0);
                gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
            },
            RenderingTarget::Blur => {
                self.program.use_program();
                self.blur_frame_buffer.bind_texture(!horizontal, 0);
                gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
            },
            RenderingTarget::Bloom => {
                self.bloom_program.use_program();
                self.frame_buffer.textures.get(0).unwrap().bind(gl::TEXTURE0);
                self.blur_frame_buffer.bind_texture(!horizontal, 1);
                gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
            }
        };
        Ok(())
    }
}

pub fn main() -> Result<(), String> {
    let mut game = Game::new(
        "Bloom",
        800,
        600,
        60,
        Vector3::new(0f32, 0f32, 0f32),
        "17.1-uniform_buffer_objects_vertex.glsl",
        "25.1-bloom.glsl",
        "17.1-uniform_buffer_objects_vertex.glsl",
        "25.1-bloom_light_fragment.glsl",
    )?;
    let cube = cube_mesh(vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/container2.png", env!("CARGO_MANIFEST_DIR")),
        },
        TextureInfo {
            id: 1,
            texture_type: TextureType::Specular,
            path: format!("{}/resource/container2_specular.png", env!("CARGO_MANIFEST_DIR")),
        },
    ]);
    for (position, color) in vec![
        (Vector3::new(0f32, 0.5f32, 1.5f32), Vector3::new(5f32, 5f32, 5f32)),
        (Vector3::new(-4f32, 0.5f32, -3f32), Vector3::new(10f32, 0f32, 0f32)),
        (Vector3::new(3f32, 0.5f32, 1f32), Vector3::new(0f32, 0f32, 15f32)),
        (Vector3::new(-0.8f32, 2.4f32, -1f32), Vector3::new(0f32, 5f32, 0f32)),
    ] {
        let point_light = PointLight::new(
            position,
            Vector3::zeros(),
            color,
            color,
            0f32,
            0f32,
            1f32,
        );
        game.spawn_light(point_light, &cube)?;
    }
    let floor = build_plane(-1f32, 12.5f32, 12.5f32, vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/wood.png", env!("CARGO_MANIFEST_DIR")),
        },
    ]);
    game.spawn_mesh(&floor, Transform::identity())?;
    for transform in vec![
        Transform {
            position: Vector3::new(0f32, 1.5f32, 0f32),
            rotation: Rotation3::identity(),
            scale: Vector3::new(0.5f32, 0.5f32, 0.5f32),
        },
        Transform {
            position: Vector3::new(2f32, 0f32, 1f32),
            rotation: Rotation3::identity(),
            scale: Vector3::new(0.5f32, 0.5f32, 0.5f32),
        },
        Transform {
            position: Vector3::new(-1f32, -1f32, 2f32),
            rotation: Rotation3::from_axis_angle(&UnitVector3::new_normalize(Vector3::new(1f32, 0f32, 1f32)), 60f32.to_radians()),
            scale: Vector3::new(1f32, 1f32, 1f32),
        },
        Transform {
            position: Vector3::new(0f32, 2.7f32, 4f32),
            rotation: Rotation3::from_axis_angle(&UnitVector3::new_normalize(Vector3::new(1f32, 0f32, 1f32)), 23f32.to_radians()),
            scale: Vector3::new(1.25f32, 1.25f32, 1.25f32),
        },
        Transform {
            position: Vector3::new(-2f32, 1f32, -3f32),
            rotation: Rotation3::from_axis_angle(&UnitVector3::new_normalize(Vector3::new(1f32, 0f32, 1f32)), 124f32.to_radians()),
            scale: Vector3::new(1f32, 1f32, 1f32),
        },
        Transform {
            position: Vector3::new(-3f32, 0f32, 0f32),
            rotation: Rotation3::identity(),
            scale: Vector3::new(0.5f32, 0.5f32, 0.5f32),
        },
    ] {
        game.spawn_mesh(&cube, transform)?;
    }
    game.spawn((Input::new(vec![InputType::Keyboard]), BloomControl {
        bloom: Keycode::V,
        blur: Keycode::C,
        brightness: Keycode::X,
        decrease_exposure: Keycode::Q,
        hdr: Keycode::Z,
        increase_exposure: Keycode::E,
    }));
    game.play_with_fps_camera(vec![Box::new(FrameBufferSystem::new()?)])?;
    Ok(())
}
