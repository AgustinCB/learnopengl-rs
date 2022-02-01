use gl;
use learnopengl::gl_function;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use learnopengl::buffer::Buffer;
use learnopengl::program::Program;
use learnopengl::shader::Shader;
use learnopengl::vertex_array::VertexArray;

const VERTEX_SHADER: &'static str = include_str!("shaders/01vertex.glsl");
const FRAGMENT_SHADER: &'static str = include_str!("shaders/01fragment.glsl");
const VERTICES: [f32; 9] = [
    -0.5f32, -0.5, 0.0,
    0.5, -0.5, 0.0,
    0.0,  0.5, 0.0
];

fn create_program(vertex: &str, fragment: &str) -> Result<Program, String> {
    let vertex = Shader::new(gl::VERTEX_SHADER, vertex)?;
    let fragment = Shader::new(gl::FRAGMENT_SHADER, fragment)?;
    let program = Program::new(vec![vertex, fragment])?;
    Ok(program)
}

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

    let program = create_program(VERTEX_SHADER, FRAGMENT_SHADER).unwrap();
    let vertex_array = VertexArray::new();
    let array_buffer = Buffer::new(gl::ARRAY_BUFFER);
    vertex_array.bind();
    array_buffer.bind();
    array_buffer.set_data(&VERTICES, gl::STATIC_DRAW);
    VertexArray::set_vertex_attrib::<f32>(gl::FLOAT, 0, 3, false);
    array_buffer.unbind();
    VertexArray::unbind();

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
        program.use_program();
        vertex_array.bind();
        gl_function!(DrawArrays(gl::TRIANGLES, 0, 3));

        window.gl_swap_window();
    }

    Ok(())
}
