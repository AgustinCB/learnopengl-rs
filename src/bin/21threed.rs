use gl;
use learnopengl::buffer::Buffer;
use learnopengl::gl_function;
use learnopengl::program::Program;
use learnopengl::shader::Shader;
use learnopengl::texture::{Texture, TextureType};
use learnopengl::vertex_array::VertexArray;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use std::ptr;
use nalgebra::{Perspective3, Rotation, Translation3, Vector3};

const VERTEX_SHADER: &'static str = include_str!("shaders/05.1-coordtexturevertex.glsl");
const FRAGMENT_SHADER: &'static str = include_str!("shaders/04.1-transformtexturefragment.glsl");
const VERTICES: [f32; 20] = [
    0.5f32, 0.5, 0.0, 1.0, 1.0, 0.5, -0.5, 0.0, 1.0, 0.0, -0.5, -0.5, 0.0, 0.0, 0.0, -0.5, 0.5,
    0.0, 0.0, 1.0,
];
const INDICES: [u32; 6] = [0, 1, 3, 1, 2, 3];

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
    let element_buffer = Buffer::new(gl::ELEMENT_ARRAY_BUFFER);
    vertex_array.bind();
    array_buffer.bind();
    array_buffer.set_data(&VERTICES, gl::STATIC_DRAW);
    element_buffer.bind();
    element_buffer.set_data(&INDICES, gl::STATIC_DRAW);
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
    let view = Translation3::new(0f32, 0f32, -3f32);
    let projection = Perspective3::new(800f32 / 600f32, 45f32.to_radians(), 0.1, 100f32);
    program.use_program();
    program.set_uniform_i1("texture1", 0);
    program.set_uniform_i1("texture2", 1);
    program.set_uniform_fv4("model", &model.to_homogeneous());
    program.set_uniform_fv4("view", &view.to_homogeneous());
    program.set_uniform_fv4("projection", &projection.to_homogeneous());

    gl_function!(ClearColor(0.3, 0.3, 0.5, 1.0));
    'gameloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'gameloop,
                _ => {}
            }
        }

        gl_function!(Clear(gl::COLOR_BUFFER_BIT));
        texture.bind(gl::TEXTURE0);
        texture2.bind(gl::TEXTURE1);
        program.use_program();
        vertex_array.bind();
        gl_function!(DrawElements(
            gl::TRIANGLES,
            6,
            gl::UNSIGNED_INT,
            ptr::null()
        ));

        window.gl_swap_window();
    }

    Ok(())
}
