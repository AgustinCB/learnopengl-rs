use gl;
use include_dir::{Dir, include_dir};
use learnopengl::buffer::Buffer;
use learnopengl::camera::Camera;
use learnopengl::gl_function;
use learnopengl::program::Program;
use learnopengl::vertex_array::VertexArray;
use nalgebra::{Perspective3, Translation3, UnitVector3, Vector3};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use learnopengl::cube::Cube;
use learnopengl::light::{DirectionalLight, Light};
use learnopengl::shader_loader::{ShaderLoader, ShaderType};
use learnopengl::texture::{Texture, TextureType};
use learnopengl::window::Window;

static SHADERS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/src/bin/shaders");

static TEXTURE_COORDS: [f32; 72] = [
    0f32, 0f32,
    1f32, 0f32,
    1f32, 1f32,
    1f32, 1f32,
    0f32, 1f32,
    0f32, 0f32,
    0f32, 0f32,
    1f32, 0f32,
    1f32, 1f32,
    1f32, 1f32,
    0f32, 1f32,
    0f32, 0f32,
    0f32, 0f32,
    1f32, 0f32,
    1f32, 1f32,
    1f32, 1f32,
    0f32, 1f32,
    0f32, 0f32,
    0f32, 0f32,
    1f32, 0f32,
    1f32, 1f32,
    1f32, 1f32,
    0f32, 1f32,
    0f32, 0f32,
    0f32, 0f32,
    1f32, 0f32,
    1f32, 1f32,
    1f32, 1f32,
    0f32, 1f32,
    0f32, 0f32,
    0f32, 0f32,
    1f32, 0f32,
    1f32, 1f32,
    1f32, 1f32,
    0f32, 1f32,
    0f32, 0f32,
];

pub fn main() -> Result<(), String> {
    let mut window = Window::new("Gouraud lighting", 800, 600).unwrap();
    let shader_loader = ShaderLoader::new(&SHADERS_DIR);
    let program = Program::new(vec![
        shader_loader.load(ShaderType::Vertex, "09.1-lightingmapsvertex.glsl").unwrap(),
        shader_loader.load(ShaderType::Fragment, "10.1-directionallight.glsl").unwrap(),
    ])?;
    let vertex_array = VertexArray::new();
    let array_buffer = Buffer::new(gl::ARRAY_BUFFER);
    let mut cube = Cube::with_normals();
    cube.add_texture(&TEXTURE_COORDS);
    vertex_array.bind();
    array_buffer.bind();
    array_buffer.set_data(cube.content(), gl::STATIC_DRAW);
    VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 0, cube.size() as _, 3, 0, false);
    VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 1, cube.size() as _, 3, 3, false);
    VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 2, cube.size() as _, 2, 6, false);

    let texture = Texture::new(TextureType::Texture2D);
    texture.bind(gl::TEXTURE0);
    texture.set_parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
    texture.set_parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
    texture.set_parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
    texture.set_parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);
    {
        let data = include_bytes!("../../resource/container2.raw");
        texture.set_image_2d(512, 512, data);
        texture.generate_mipmap();
    }

    let specular_texture = Texture::new(TextureType::Texture2D);
    specular_texture.bind(gl::TEXTURE1);
    specular_texture.set_parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
    specular_texture.set_parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
    specular_texture.set_parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
    specular_texture.set_parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);
    {
        let data = include_bytes!("../../resource/container2_specular.png.raw");
        specular_texture.set_image_2d(512, 512, data);
        specular_texture.generate_mipmap();
    }

    let mut fov = 45f32;

    let cube_positions: [Vector3<f32>; 10] = [
        Vector3::new(0.0f32, 0.0f32, 0.0f32),
        Vector3::new(2.0f32, 5.0f32, -15.0f32),
        Vector3::new(-1.5f32, -2.2f32, -2.5f32),
        Vector3::new(-3.8f32, -2.0f32, -12.3f32),
        Vector3::new(2.4f32, -0.4f32, -3.5f32),
        Vector3::new(-1.7f32, 3.0f32, -7.5f32),
        Vector3::new(1.3f32, -2.0f32, -2.5f32),
        Vector3::new(1.5f32, 2.0f32, -2.5f32),
        Vector3::new(1.5f32, 0.2f32, -1.5f32),
        Vector3::new(-1.3f32, 1.0f32, -1.5f32),
    ];
    let mut camera = Camera::new(
        Vector3::new(0.0f32, 0f32, 3f32),
        Vector3::new(0f32, 0f32, -1f32),
        Vector3::y_axis(),
    );
    let mut yaw = -90f32;
    let mut pitch = 0f32;
    let directional_light = DirectionalLight::new(
        UnitVector3::new_normalize(Vector3::new(-0.2f32, -1f32, -0.3f32)),
        Vector3::new(0.2f32, 0.2f32, 0.2f32),
        Vector3::new(0.5f32, 0.5f32, 0.5f32),
        Vector3::new(1f32, 1f32, 1f32),
    );
    program.use_program();
    program.set_uniform_i1("material.diffuse", 0);
    program.set_uniform_i1("material.specular", 1);
    program.set_uniform_f1("material.shininess", 32f32);
    directional_light.set_light_in_program(&program, "light");

    window.start_timer();
    gl_function!(Enable(gl::DEPTH_TEST));
    gl_function!(ClearColor(0.3, 0.3, 0.5, 1.0));
    'gameloop: loop {
        let delta_time = window.delta_time();
        let camera_speed = 0.01f32 * delta_time;
        for event in window.events() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'gameloop,
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    camera.move_forward(camera_speed);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    camera.move_forward(-camera_speed);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    camera.move_right(camera_speed);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    camera.move_right(-camera_speed);
                }
                Event::MouseMotion { xrel, yrel, .. } => {
                    let sensitivity = 0.05f32;
                    let xoffset = xrel as f32 * sensitivity;
                    let yoffset = yrel as f32 * sensitivity;
                    yaw += xoffset;
                    pitch += yoffset;
                    pitch = pitch.clamp(-89f32, 89f32);
                    camera.set_front(yaw, pitch);
                }
                Event::MouseWheel { y, .. } => {
                    fov -= y as f32;
                    fov = fov.clamp(1f32, 45f32);
                }
                _ => {}
            }
        }

        gl_function!(Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
        vertex_array.bind();
        let look_at = camera.look_at_matrix();
        let projection = Perspective3::new(800f32 / 600f32, fov.to_radians(), 0.1, 100f32);
        texture.bind(gl::TEXTURE0);
        specular_texture.bind(gl::TEXTURE1);
        program.use_program();
        program.set_uniform_matrix4("view", &look_at);
        program.set_uniform_matrix4("projection", &projection.to_homogeneous());
        program.set_uniform_v3("viewPos", camera.position());
        for cube in cube_positions.iter() {
            let t = Translation3::from(cube.data.0[0]).to_homogeneous();
            program.set_uniform_matrix4("model", &t);
            gl_function!(DrawArrays(gl::TRIANGLES, 0, 36,));
        }

        window.swap_buffers();
        window.delay(1000/60);
    }

    Ok(())
}
