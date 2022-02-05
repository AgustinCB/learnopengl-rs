#[macro_export]

macro_rules! gl_function {
    ($a:ident($($b:tt)*)) => {
        unsafe {
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
