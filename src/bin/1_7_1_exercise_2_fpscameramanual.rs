use gl;
use learnopengl::buffer::Buffer;
use learnopengl::camera::Camera;
use learnopengl::gl_function;
use learnopengl::program::Program;
use learnopengl::shader::Shader;
use learnopengl::texture::{Texture, TextureType};
use learnopengl::vertex_array::VertexArray;
use nalgebra::{Perspective3, Rotation, Rotation3, Translation3, Vector3};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

const VERTEX_SHADER: &'static str = include_str!("shaders/05.1-coordtexturevertex.glsl");
const FRAGMENT_SHADER: &'static str = include_str!("shaders/04.1-transformtexturefragment.glsl");
const VERTICES: [f32; 180] = [
    -0.5f32, -0.5f32, -0.5f32, 0.0f32, 0.0f32, 0.5f32, -0.5f32, -0.5f32, 1.0f32, 0.0f32, 0.5f32,
    0.5f32, -0.5f32, 1.0f32, 1.0f32, 0.5f32, 0.5f32, -0.5f32, 1.0f32, 1.0f32, -0.5f32, 0.5f32,
    -0.5f32, 0.0f32, 1.0f32, -0.5f32, -0.5f32, -0.5f32, 0.0f32, 0.0f32, -0.5f32, -0.5f32, 0.5f32,
    0.0f32, 0.0f32, 0.5f32, -0.5f32, 0.5f32, 1.0f32, 0.0f32, 0.5f32, 0.5f32, 0.5f32, 1.0f32,
    1.0f32, 0.5f32, 0.5f32, 0.5f32, 1.0f32, 1.0f32, -0.5f32, 0.5f32, 0.5f32, 0.0f32, 1.0f32,
    -0.5f32, -0.5f32, 0.5f32, 0.0f32, 0.0f32, -0.5f32, 0.5f32, 0.5f32, 1.0f32, 0.0f32, -0.5f32,
    0.5f32, -0.5f32, 1.0f32, 1.0f32, -0.5f32, -0.5f32, -0.5f32, 0.0f32, 1.0f32, -0.5f32, -0.5f32,
    -0.5f32, 0.0f32, 1.0f32, -0.5f32, -0.5f32, 0.5f32, 0.0f32, 0.0f32, -0.5f32, 0.5f32, 0.5f32,
    1.0f32, 0.0f32, 0.5f32, 0.5f32, 0.5f32, 1.0f32, 0.0f32, 0.5f32, 0.5f32, -0.5f32, 1.0f32,
    1.0f32, 0.5f32, -0.5f32, -0.5f32, 0.0f32, 1.0f32, 0.5f32, -0.5f32, -0.5f32, 0.0f32, 1.0f32,
    0.5f32, -0.5f32, 0.5f32, 0.0f32, 0.0f32, 0.5f32, 0.5f32, 0.5f32, 1.0f32, 0.0f32, -0.5f32,
    -0.5f32, -0.5f32, 0.0f32, 1.0f32, 0.5f32, -0.5f32, -0.5f32, 1.0f32, 1.0f32, 0.5f32, -0.5f32,
    0.5f32, 1.0f32, 0.0f32, 0.5f32, -0.5f32, 0.5f32, 1.0f32, 0.0f32, -0.5f32, -0.5f32, 0.5f32,
    0.0f32, 0.0f32, -0.5f32, -0.5f32, -0.5f32, 0.0f32, 1.0f32, -0.5f32, 0.5f32, -0.5f32, 0.0f32,
    1.0f32, 0.5f32, 0.5f32, -0.5f32, 1.0f32, 1.0f32, 0.5f32, 0.5f32, 0.5f32, 1.0f32, 0.0f32,
    0.5f32, 0.5f32, 0.5f32, 1.0f32, 0.0f32, -0.5f32, 0.5f32, 0.5f32, 0.0f32, 0.0f32, -0.5f32,
    0.5f32, -0.5f32, 0.0f32, 1.0f32,
];

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let attrs = video_subsystem.gl_attr();

    attrs.set_context_major_version(3);
    attrs.set_context_minor_version(3);
    attrs.set_context_profile(GLProfile::Core);
    #[cfg(target_os = "macos")]
    attrs.set_context_flags().forward_compatible().set();

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let _opengl = window.gl_create_context().unwrap();
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);
    let mut event_pump = sdl_context.event_pump()?;

    let program = Program::new(vec![
        Shader::new(gl::VERTEX_SHADER, VERTEX_SHADER).unwrap(),
        Shader::new(gl::FRAGMENT_SHADER, FRAGMENT_SHADER).unwrap(),
    ])?;
    let vertex_array = VertexArray::new();
    let array_buffer = Buffer::new(gl::ARRAY_BUFFER);
    vertex_array.bind();
    array_buffer.bind();
    array_buffer.set_data(&VERTICES, gl::STATIC_DRAW);
    VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 0, 5, 3, 0, false);
    VertexArray::set_vertex_attrib_with_padding::<f32>(gl::FLOAT, 1, 5, 2, 3, false);

    let texture = Texture::new(TextureType::Texture2D);
    let texture2 = Texture::new(TextureType::Texture2D);
    texture.bind(gl::TEXTURE0);
    texture.set_parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
    texture.set_parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
    texture.set_parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
    texture.set_parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);
    {
        let data = include_bytes!("../../resource/container.raw");
        texture.set_image_2d(512, 512, data);
        texture.generate_mipmap();
    }
    texture2.bind(gl::TEXTURE0);
    texture2.set_parameter(gl::TEXTURE_WRAP_S, gl::REPEAT);
    texture2.set_parameter(gl::TEXTURE_WRAP_T, gl::REPEAT);
    texture2.set_parameter(gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR);
    texture2.set_parameter(gl::TEXTURE_MAG_FILTER, gl::LINEAR);
    {
        let data = include_bytes!("../../resource/face1.raw");
        texture2.set_image_2d(476, 476, data);
        texture2.generate_mipmap();
    }
    let model = Rotation::from_axis_angle(&Vector3::x_axis(), -55f32.to_radians());
    let mut fov = 45f32;
    program.use_program();
    program.set_uniform_i1("texture1", 0);
    program.set_uniform_i1("texture2", 1);

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
    let rotation_vector = Vector3::new(1f32, 0.3f32, 0.5f32);
    let mut camera = Camera::new(
        Vector3::new(0.0f32, 0f32, 3f32),
        Vector3::new(0f32, 0f32, -1f32),
        Vector3::y_axis(),
    );
    let mut delta_time = 0f32;
    let mut last_frame = 0f32;
    let mut yaw = -90f32;
    let mut pitch = 0f32;

    gl_function!(Enable(gl::DEPTH_TEST));
    gl_function!(ClearColor(0.3, 0.3, 0.5, 1.0));
    'gameloop: loop {
        let mut sdl_timer = sdl_context.timer().unwrap();
        let ticks = sdl_timer.ticks() as f32;
        delta_time = ticks - last_frame;
        last_frame = ticks;
        let camera_speed = 0.01f32 * delta_time;
        for event in event_pump.poll_iter() {
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
                    camera.ground();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    camera.move_forward(-camera_speed);
                    camera.ground();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    camera.move_right(camera_speed);
                    camera.ground();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    camera.move_right(-camera_speed);
                    camera.ground();
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
        texture.bind(gl::TEXTURE0);
        texture2.bind(gl::TEXTURE1);
        program.use_program();
        vertex_array.bind();
        let look_at = camera.manual_look_at_matrix();
        program.set_uniform_matrix4("view", &look_at);
        let projection = Perspective3::new(800f32 / 600f32, fov.to_radians(), 0.1, 100f32);
        program.set_uniform_matrix4("projection", &projection.to_homogeneous());
        for (i, cube) in cube_positions.iter().enumerate() {
            let angle = 20f32 * i as f32;
            let r = Rotation3::new(rotation_vector * (angle / rotation_vector.magnitude()))
                .to_homogeneous();
            let t = Translation3::from(cube.data.0[0]).to_homogeneous();
            program.set_uniform_matrix4("model", &(t * r * model.to_homogeneous()));
            gl_function!(DrawArrays(gl::TRIANGLES, 0, 36,));
        }

        window.gl_swap_window();
        sdl_timer.delay(100);
    }

    Ok(())
}
