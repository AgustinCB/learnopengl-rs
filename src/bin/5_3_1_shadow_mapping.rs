use hecs::World;
use nalgebra::{Matrix4, Orthographic3, Rotation3, UnitVector3, Vector3};
use russimp::texture::TextureType;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use learnopengl::buffer::Buffer;
use learnopengl::cube::cube_mesh;
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
use learnopengl::vertex_array::VertexArray;

const QUAD_VERTICES: [f32; 24] = [
    -1f32, 1f32, 0f32, 1f32,
    -1f32, -1f32, 0f32, 0f32,
    1f32, -1f32, 1f32, 0f32,
    -1f32, 1f32, 0f32, 1f32,
    1f32, -1f32, 1f32, 0f32,
    1f32, 1f32, 1f32, 1f32,
];

struct DebugControl(Keycode);

struct DepthMapSystem {
    frame_buffer: FrameBuffer,
    program: Program,
    quad_program: Program,
    space_matrix: Matrix4<f32>,
    vao: VertexArray,
    _vbo: Buffer,
}

impl DepthMapSystem {
    pub fn new(light_position: Vector3<f32>) -> Result<DepthMapSystem, String> {
        let frame_buffer = FrameBuffer::depth_buffer(1024, 1024);
        let program = Program::new(vec![
            Shader::new(ShaderType::Vertex as _, include_str!("shaders/21.1-depth_calculation_vertex.glsl"))?,
            Shader::new(ShaderType::Fragment as _, include_str!("shaders/21.1-depth_calculation_fragment.glsl"))?
        ])?;
        let quad_program = Program::new(vec![
            Shader::new(ShaderType::Vertex as _, include_str!("shaders/21.1-debug_quad.glsl"))?,
            Shader::new(ShaderType::Fragment as _, include_str!("shaders/21.1-debug_quad_fragment.glsl"))?
        ])?;
        quad_program.use_program();
        quad_program.set_uniform_f1("near_plane", 1f32);
        quad_program.set_uniform_f1("far_plane", 7.5f32);
        quad_program.set_uniform_i1("depth_map", 0);
        program.use_program();
        let view = Rotation3::look_at_rh(&light_position, &Vector3::new(0f32, 1f32, 0f32));
        let projection = Orthographic3::new(-10f32, 10f32, -10f32, 10f32, 1f32, 7.5f32);
        let space_matrix = view.to_homogeneous() * projection.to_homogeneous();
        program.set_uniform_matrix4("space_matrix", &space_matrix);
        let vao = VertexArray::new();
        let vbo = Buffer::new(gl::ARRAY_BUFFER);
        vao.bind();
        vbo.bind();
        vbo.set_data(&QUAD_VERTICES, gl::STATIC_DRAW);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 0, 4, 2, 0, false);
        VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 1, 4, 2, 2, false);
        Ok(DepthMapSystem {
            frame_buffer,
            program,
            quad_program,
            space_matrix,
            vao,
            _vbo: vbo,
        })
    }

    fn render_scene(&self, world: &mut World) -> Result<(), String> {
        self.program.use_program();
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

impl System for DepthMapSystem {
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
                    name: "lightSpaceMatrix",
                    value: UniformValue::Matrix(self.space_matrix),
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
        gl_function!(CullFace(gl::FRONT));
        self.render_scene(world)?;
        gl_function!(CullFace(gl::BACK));
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
                        gl_function!(DrawArrays(gl::TRIANGLES, 0, 6));
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
        "21.1-shadow_mapping_vertex.glsl",
        "21.1-shadow_mapping_fragment.glsl",
        "17.1-uniform_buffer_objects_vertex.glsl",
        "09.1-lightfragment.glsl",
        4
    )?;
    let light_cube = cube_mesh(vec![]);
    let position = Vector3::new(-2f32, 4f32, -1f32);
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
        position: Vector3::new(0f32, 1.5f32, -5.5f32),
        scale: Vector3::new(0.5f32, 0.5f32, 0.5f32),
        rotation: Rotation3::identity(),
    })?;
    game.spawn_mesh(&cube, Transform {
        position: Vector3::new(1f32, -0.25f32, -6f32),
        scale: Vector3::new(0.5f32, 0.5f32, 0.5f32),
        rotation: Rotation3::identity(),
    })?;
    game.spawn_mesh(&cube, Transform {
        position: Vector3::new(-1f32, -0.25f32, -7f32),
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
    game.play_with_fps_camera(vec![Box::new(DepthMapSystem::new(position)?)])?;
    Ok(())
}
