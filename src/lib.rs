#[macro_export]
macro_rules! gl_function {
    ($a:ident($($b:tt)*)) => {
        unsafe {
            log::trace!("gl{}({})", stringify!($a), stringify!($($b)*));
            let return_value = gl::$a($($b)*);
            #[cfg(debug_assertions)]
            {
                let error_code = gl::GetError();
                if error_code != gl::NO_ERROR {
                    log::error!("ERROR CODE {}", error_code);
                    std::process::exit(error_code as i32);
                }
            }
            return_value
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
pub mod loader;
pub mod plane;