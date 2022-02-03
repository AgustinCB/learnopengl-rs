use crate::gl_function;
use gl;
use std::mem::transmute;

#[derive(Clone, Copy)]
pub enum TextureType {
    Texture1D = gl::TEXTURE_1D as isize,
    Texture2D = gl::TEXTURE_2D as isize,
    Texture3D = gl::TEXTURE_3D as isize,
}

pub struct Texture(pub(crate) gl::types::GLuint, gl::types::GLenum, TextureType);

impl Texture {
    pub fn new(texture_type: TextureType) -> Texture {
        let mut texture = 0 as gl::types::GLuint;
        gl_function!(GenTextures(1, &mut texture));
        Texture(texture, texture_type as u32, texture_type)
    }

    pub fn bind(&self, unit: gl::types::GLenum) {
        gl_function!(ActiveTexture(unit));
        gl_function!(BindTexture(self.1, self.0));
    }

    pub fn generate_mipmap(&self) {
        gl_function!(GenerateMipmap(self.1));
    }

    pub fn set_image_2d(&self, width: u32, height: u32, data: &[u8]) {
        match self.2 {
            TextureType::Texture2D => gl_function!(TexImage2D(
                self.1,
                0,
                gl::RGB as _,
                width as _,
                height as _,
                0,
                gl::RGB as _,
                gl::UNSIGNED_BYTE,
                transmute(&(data[0]) as *const u8)
            )),
            _ => unimplemented!(),
        }
    }

    pub fn set_parameter(&self, parameter: gl::types::GLenum, value: gl::types::GLenum) {
        gl_function!(TexParameteri(self.1, parameter, value as i32));
    }
}
