use std::cell::RefCell;
use hecs::World;
use nalgebra::{IsometryMatrix3, Perspective3, Point3, Rotation3, UnitVector3, Vector3};
use russimp::texture::TextureType;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use learnopengl::buffer::Buffer;
use learnopengl::cube::{cube_mesh, VERTICES};
use learnopengl::ecs::components::{Mesh, Shader as MeshShader, TextureInfo, Transform, ExtraUniform, UniformValue, Input};
use learnopengl::ecs::systems::input::InputType;
use learnopengl::ecs::systems::system::System;
use learnopengl::frame_buffer::FrameBuffer;
use learnopengl::game::Game;
use learnopengl::gl_function;
use learnopengl::light::PointLight;
use learnopengl::plane::build_plane;
use learnopengl::program::Program;
use learnopengl::shader::Shader;
use learnopengl::shader_loader::ShaderType;
use learnopengl::texture::{Texture, TextureType as OpenGLTextureType};
use learnopengl::vertex_array::VertexArray;

struct DebugControl(Keycode);

struct MovingPointLight {
    sin_offset: f32,
    total_ticks: RefCell<f32>,
    overall: f32,
}

impl System for MovingPointLight {
    fn name(&self) -> &str {
        "Moving Point Light"
    }

    fn start(&self, _world: &mut World) -> Result<(), String> {
        Ok(())
    }

    fn early_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }

    fn update(&self, world: &mut World, delta_time: f32) -> Result<(), String> {
        let total_ticks = *self.total_ticks.borrow() + delta_time;
        self.total_ticks.replace(total_ticks);
        if let Some((_, light)) = world.query_mut::<&mut PointLight>().into_iter().next() {
            *light.position.get_mut(2).unwrap() = ((total_ticks * self.sin_offset).sin()) * self.overall;
            light.update_model();
        }
        Ok(())
    }

    fn late_update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }
}

struct DepthCubeMapSystem {
    frame_buffer: FrameBuffer,
    program: Program,
    quad_program: Program,
    vao: VertexArray,
    _vbo: Buffer,
}

fn create_texture() -> Texture {
    let cube_texture = Texture::new(OpenGLTextureType::CubeMap);
    cube_texture.just_bind();
    for i in 0..6 {
        cube_texture.alloc_depth_cube_map_face(i, 1024, 1024);
    }
    gl_function!(TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _));
    gl_function!(TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _));
    gl_function!(TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as _));
    gl_function!(TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _));
    gl_function!(TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _));
    cube_texture
}

impl DepthCubeMapSystem {
    pub fn new() -> Result<DepthCubeMapSystem, String> {
        let frame_buffer = FrameBuffer::depth_cubemap_with_texture(create_texture());
        let program = Program::new(vec![
            Shader::new(ShaderType::Vertex as _, include_str!("shaders/22.1-depth_calculation_vertex.glsl"))?,
            Shader::new(ShaderType::Geometry as _, include_str!("shaders/22.1-depth_calculation_geometry.glsl"))?,
            Shader::new(ShaderType::Fragment as _, include_str!("shaders/22.1-depth_calculation_fragment.glsl"))?
        ])?;
        let quad_program = Program::new(vec![
            Shader::new(ShaderType::Vertex as _, include_str!("shaders/22.1-debug_quad.glsl"))?,
            Shader::new(ShaderType::Fragment as _, include_str!("shaders/22.1-debug_quad_fragment.glsl"))?
        ])?;
        let near = 1f32;
        let far = 25f32;
        quad_program.use_program();
        quad_program.set_uniform_f1("near_plane", near);
        quad_program.set_uniform_f1("far_plane", far);
        quad_program.set_uniform_i1("depth_map", 0);
        quad_program.bind_uniform_block("Matrices", 0);
        program.use_program();
        program.set_uniform_f1("far_plane", far);
        let vao = VertexArray::new();
        let vbo = Buffer::new(gl::ARRAY_BUFFER);
        vao.bind();
        vbo.bind();
        let cube_vertices = VERTICES.iter().map(|f| *f * 2f32).collect::<Vec<f32>>();
        vbo.set_data(&cube_vertices, gl::STATIC_DRAW);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 0, 3, 3, 0, false);
        Ok(DepthCubeMapSystem {
            frame_buffer,
            program,
            quad_program,
            vao,
            _vbo: vbo,
        })
    }

    fn render_scene(&self, world: &mut World) -> Result<(), String> {
        self.program.use_program();
        if let Some((_, light)) = world.query_mut::<&PointLight>().into_iter().next() {
            let projection = Perspective3::new(1f32, 90f32.to_radians(), 1f32, 25f32);
            let position = Point3::from(light.position);
            for (i, (direction, up)) in vec![
                (Vector3::new(1f32, 0f32, 0f32), Vector3::new(0f32, -1f32, 0f32)),
                (Vector3::new(-1f32, 0f32, 0f32), Vector3::new(0f32, -1f32, 0f32)),
                (Vector3::new(0f32, 1f32, 0f32), Vector3::new(0f32, 0f32, 1f32)),
                (Vector3::new(0f32, -1f32, 0f32), Vector3::new(0f32, 0f32, -1f32)),
                (Vector3::new(0f32, 0f32, 1f32), Vector3::new(0f32, -1f32, 0f32)),
                (Vector3::new(0f32, 0f32, -1f32), Vector3::new(0f32, -1f32, 0f32)),
            ].into_iter().enumerate() {
                let target = Point3::from(light.position + direction);
                let view = IsometryMatrix3::look_at_rh(&position, &target, &up);
                let space_matrix = projection.to_homogeneous() * view.to_homogeneous();
                self.program.set_uniform_matrix4(&format!("shadowMatrices[{}]", i), &space_matrix);
            }
            self.program.set_uniform_v3("lightPos", light.position);
        }
        for (_e, (mesh, shader, transform)) in world.query_mut::<(&Mesh, &MeshShader, &Transform)>() {
            self.program.set_uniform_matrix4("model", &transform.get_model_matrix());
            let n_vertices = mesh.vertices.len();
            shader.vertex_array.bind();
            gl_function!(DrawArrays(gl::TRIANGLES, 0, n_vertices as i32));
            VertexArray::unbind();
        }
        Ok(())
    }
}

impl System for DepthCubeMapSystem {
    fn name(&self) -> &str {
        "Frame Buffer"
    }

    fn start(&self, world: &mut World) -> Result<(), String> {
        let mut es = vec![];
        for (e, _mesh) in world.query::<&Mesh>().iter() {
            es.push(e);
        }
        for e in es {
            world.insert_one(e, vec![
                ExtraUniform {
                    name: "far_plane",
                    value: UniformValue::Float(25.0),
                },
                ExtraUniform {
                    name: "shadowMap",
                    value: UniformValue::Texture(8),
                },
            ]).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn early_update(&self, world: &mut World, _delta_time: f32) -> Result<(), String> {
        gl_function!(Clear(gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT));
        gl_function!(Viewport(0, 0, 1024, 1024));
        self.frame_buffer.bind();
        gl_function!(Clear(gl::DEPTH_BUFFER_BIT));
        // gl_function!(CullFace(gl::FRONT));
        self.render_scene(world)?;
        // gl_function!(CullFace(gl::BACK));
        FrameBuffer::unbind();
        gl_function!(Viewport(0, 0, 800, 600));
        self.frame_buffer.texture.bind(gl::TEXTURE8);
        Ok(())
    }

    fn update(&self, _world: &mut World, _delta_time: f32) -> Result<(), String> {
        Ok(())
    }

    fn late_update(&self, world: &mut World, _delta_time: f32) -> Result<(), String> {
        for (_e, (debug_control, input)) in world.query_mut::<(&DebugControl, &Input)>() {
            for ke in input.events.iter() {
                match ke {
                    Event::KeyDown { keycode: Some(k), ..} if k == &debug_control.0 => {
                        self.quad_program.use_program();
                        self.vao.bind();
                        self.frame_buffer.texture.bind(gl::TEXTURE0);
                        gl_function!(Disable(gl::DEPTH_TEST));
                        gl_function!(DrawArrays(gl::TRIANGLES, 0, 36));
                        gl_function!(Enable(gl::DEPTH_TEST));
                    }
                    _ => {}
                }
            }
        }
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
        "22.1-point_shadow_fragment.glsl",
        "17.1-uniform_buffer_objects_vertex.glsl",
        "09.1-lightfragment.glsl",
        4
    )?;
    let light_cube = cube_mesh(vec![]);
    let position = Vector3::new(-2f32, 1.5f32, -1f32);
    let point_light = PointLight::new(
        position,
        Vector3::new(0.3f32 * 0.3f32, 0.3f32 * 0.3f32, 0.3f32 * 0.3f32),
        Vector3::new(0.3f32, 0.3f32, 0.3f32),
        Vector3::new(0.3f32, 0.3f32, 0.3f32),
        1f32,
        0f32,
        0f32,
    );
    game.spawn_light(point_light, &light_cube)?;
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
    game.spawn_mesh(&cube, Transform {
        position: Vector3::new(0f32, 1f32, 0f32),
        scale: Vector3::new(0.5f32, 0.5f32, 0.5f32),
        rotation: Rotation3::identity(),
    })?;
    game.spawn_mesh(&cube, Transform {
        position: Vector3::new(1f32, -0.25f32, -1f32),
        scale: Vector3::new(0.5f32, 0.5f32, 0.5f32),
        rotation: Rotation3::identity(),
    })?;
    game.spawn_mesh(&cube, Transform {
        position: Vector3::new(-1f32, -0.25f32, -2f32),
        scale: Vector3::new(0.25f32, 0.25f32, 0.25f32),
        rotation: Rotation3::from_axis_angle(&UnitVector3::new_normalize(Vector3::new(1f32, 0f32, 1f32)), 60f32.to_radians()),
    })?;
    let floor = build_plane(-0.5f32, 25f32, 25f32, vec![
        TextureInfo {
            id: 0,
            texture_type: TextureType::Diffuse,
            path: format!("{}/resource/wood.png", env!("CARGO_MANIFEST_DIR")),
        },
        TextureInfo {
            id: 1,
            texture_type: TextureType::Specular,
            path: format!("{}/resource/wood.png", env!("CARGO_MANIFEST_DIR")),
        }
    ]);
    game.spawn_mesh(&floor, Transform::identity())?;
    game.spawn((DebugControl(Keycode::E), Input::new(vec![InputType::Keyboard])));
    game.play_with_fps_camera(vec![Box::new(DepthCubeMapSystem::new()?), Box::new(MovingPointLight { sin_offset: 0.001f32, total_ticks: RefCell::new(0f32), overall: 3f32, })])?;
    Ok(())
}
