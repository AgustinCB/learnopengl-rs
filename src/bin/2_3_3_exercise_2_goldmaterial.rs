use gl;
use include_dir::{Dir, include_dir};
use learnopengl::buffer::Buffer;
use learnopengl::camera::Camera;
use learnopengl::gl_function;
use learnopengl::program::Program;
use learnopengl::vertex_array::VertexArray;
use nalgebra::{Perspective3, Scale3, Translation3, Vector3};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use learnopengl::cube::Cube;
use learnopengl::shader_loader::{ShaderLoader, ShaderType};
use learnopengl::window::Window;

static SHADERS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/src/bin/shaders");

pub fn main() -> Result<(), String> {
    let mut window = Window::new("Gouraud lighting", 800, 600).unwrap();
    let shader_loader = ShaderLoader::new(&SHADERS_DIR);
    let program = Program::new(vec![
        shader_loader.load(ShaderType::Vertex, "07.1-basiclightvertex.glsl").unwrap(),
        shader_loader.load(ShaderType::Fragment, "08.1-material.glsl").unwrap(),
    ])?;
    let light_program = Program::new(vec![
        shader_loader.load(ShaderType::Vertex, "07.1-basiclightvertex.glsl").unwrap(),
        shader_loader.load(ShaderType::Fragment, "08.2-lightfragment.glsl").unwrap(),
    ])?;
    let vertex_array = VertexArray::new();
    let array_buffer = Buffer::new(gl::ARRAY_BUFFER);
    let cube = Cube::with_normals();
    vertex_array.bind();
    array_buffer.bind();
    array_buffer.set_data(cube.content(), gl::STATIC_DRAW);
    VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 0, cube.size() as _, 3, 0, false);
    VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 1, cube.size() as _, 3, 3, false);

    let mut fov = 45f32;

    let cube_positions: [Vector3<f32>; 2] = [
        Vector3::new(0.0f32, 0.0f32, 0.0f32),
        Vector3::new(-2f32, 1.0f32, -1.5f32),
    ];
    let scale: [f32; 2] = [ 1f32, 0.5f32 ];
    let mut camera = Camera::new(
        Vector3::new(0.0f32, 0f32, 3f32),
        Vector3::new(0f32, 0f32, -1f32),
        Vector3::y_axis(),
    );
    let mut yaw = -90f32;
    let mut pitch = 0f32;
    program.use_program();
    program.set_uniform_v3("material.ambient", Vector3::new(0.24725f32, 0.1995f32, 0.0745f32));
    program.set_uniform_v3("material.diffuse", Vector3::new(0.22648f32, 0.60648f32, 0.22648f32));
    program.set_uniform_v3("material.specular", Vector3::new(0.628281f32, 0.555802f32, 0.366065f32));
    program.set_uniform_f1("material.shininess", 32f32);
    program.set_uniform_v3("light.specular", Vector3::new(1f32, 1f32, 1f32));

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
        let light_color = Vector3::new((delta_time * 2f32).sin(), (delta_time * 0.7f32).sin(), (delta_time * 1.3f32).sin());
        let diffuse_color = light_color * 0.5f32;
        let ambient_color = diffuse_color * 0.2f32;
        for ((cube, scale), program) in cube_positions
            .iter()
            .zip(scale)
            .zip(&[&program, &light_program]) {
            program.use_program();
            program.set_uniform_matrix4("view", &look_at);
            program.set_uniform_matrix4("projection", &projection.to_homogeneous());
            program.set_uniform_v3("objectColor", Vector3::new(1f32, 0.5f32, 0.31f32));
            program.set_uniform_v3("lightColor", light_color);
            program.set_uniform_v3("light.position", cube_positions[1]);
            program.set_uniform_v3("light.ambient", ambient_color);
            program.set_uniform_v3("light.diffuse", diffuse_color);
            program.set_uniform_v3("viewPos", camera.position());
            let s = Scale3::new(scale, scale, scale).to_homogeneous();
            let t = Translation3::from(cube.data.0[0]).to_homogeneous();
            program.set_uniform_matrix4("model", &(s * t));
            gl_function!(DrawArrays(gl::TRIANGLES, 0, 36,));
        }

        window.swap_buffers();
        window.delay(1000/60);
    }

    Ok(())
}
