#[macro_export]
macro_rules! gl_function {
    ($a:ident($($b:tt)*)) => {
        unsafe {
            log::trace!("gl{}({})", stringify!($a), stringify!($($b)*));
            gl::$a($($b)*)
        }
    };
}

pub mod buffer;
pub mod camera;
pub mod program;
pub mod shader;
pub mod texture;
pub mod vertex_array;
pub mod cube;
pub mod window;
pub mod shader_loader;
pub mod light;
pub mod ecs;
pub mod game;