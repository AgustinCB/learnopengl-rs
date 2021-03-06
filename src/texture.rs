use crate::gl_function;
use gl;
use std::mem::transmute;
use std::ptr;
use image::ColorType;
use itertools::Itertools;

#[derive(Clone, Copy, Debug)]
pub enum TextureType {
    Texture1D = gl::TEXTURE_1D as isize,
    Texture2D = gl::TEXTURE_2D as isize,
    Texture3D = gl::TEXTURE_3D as isize,
    CubeMap = gl::TEXTURE_CUBE_MAP as isize,
    Texture2DMultisample = gl::TEXTURE_2D_MULTISAMPLE as isize,
}

#[derive(Clone, Copy)]
pub enum TextureFormat {
    FloatingPoint,
    UnsignedByte,
    UnsignedByteWithAlpha,
    Grey,
}

#[derive(Debug)]
pub struct Texture(pub(crate) gl::types::GLuint, pub(crate) gl::types::GLenum, TextureType);

impl Texture {
    pub fn multiple(texture_type: TextureType, size: usize) -> Vec<Texture> {
        let mut texture_resources = [0].repeat(size);
        gl_function!(GenTextures(size as i32, texture_resources.as_mut_ptr()));
        texture_resources.into_iter()
            .map(|r| Texture(r, texture_type as u32, texture_type))
            .collect_vec()
    }

    pub fn new(texture_type: TextureType) -> Texture {
        let mut texture = 0 as gl::types::GLuint;
        gl_function!(GenTextures(1, &mut texture));
        Texture(texture, texture_type as u32, texture_type)
    }

    pub fn unbind(&self) {
        gl_function!(BindTexture(self.1, 0));
    }

    pub fn bind_as(&self, unit: gl::types::GLenum, texture_type: gl::types::GLenum) {
        gl_function!(ActiveTexture(unit));
        gl_function!(BindTexture(texture_type, self.0));
    }

    pub fn bind(&self, unit: gl::types::GLenum) {
        gl_function!(ActiveTexture(unit));
        self.just_bind();
    }

    pub fn just_bind(&self) {
        gl_function!(BindTexture(self.1, self.0));
    }

    pub fn generate_mipmap(&self) {
        gl_function!(GenerateMipmap(self.1));
    }

    pub fn alloc_depth_cube_map_face(&self, face: u32, width: usize, height: usize) {
        gl_function!(TexImage2D(
            gl::TEXTURE_CUBE_MAP_POSITIVE_X + face,
            0,
            gl::DEPTH_COMPONENT as _,
            width as _,
            height as _,
            0,
            gl::DEPTH_COMPONENT as _,
            gl::FLOAT,
            ptr::null(),
        ));
    }

    pub fn set_cube_map_face(&self, face: u32, width: usize, height: usize, data: &[u8]) {
        gl_function!(TexImage2D(
            gl::TEXTURE_CUBE_MAP_POSITIVE_X + face,
            0,
            gl::RGBA as _,
            width as _,
            height as _,
            0,
            gl::RGBA as _,
            gl::UNSIGNED_BYTE,
            transmute(&(data[0]) as *const u8)
        ));
    }

    pub fn set_image_2d_with_type(&self, width: u32, height: u32, data: &[u8], color_type: ColorType) -> Result<(), String> {
        let gl_type = match color_type {
            ColorType::Rgb8 => Ok(gl::RGB),
            ColorType::Rgba8 => Ok(gl::RGBA),
            t => Err(format!("Unsupported format {:?}", &t)),
        }?;
        match self.2 {
            TextureType::Texture2D => gl_function!(TexImage2D(
                self.1,
                0,
                gl_type as _,
                width as _,
                height as _,
                0,
                gl_type as _,
                gl::UNSIGNED_BYTE,
                transmute(&(data[0]) as *const u8)
            )),
            _ => unimplemented!(),
        };
        Ok(())
    }

    pub fn set_image_2d(&self, width: u32, height: u32, data: &[u8]) {
        match self.2 {
            TextureType::Texture2D => gl_function!(TexImage2D(
                self.1,
                0,
                gl::RGBA as _,
                width as _,
                height as _,
                0,
                gl::RGBA as _,
                gl::UNSIGNED_BYTE,
                transmute(&(data[0]) as *const u8)
            )),
            _ => unimplemented!(),
        }
    }

    pub fn set_image_2d_with_format<T>(&self, width: u32, height: u32, data: &[T], format: TextureFormat) {
        match (self.2, format) {
            (TextureType::Texture2D, TextureFormat::UnsignedByteWithAlpha) => gl_function!(TexImage2D(
                self.1,
                0,
                gl::RGBA as _,
                width as _,
                height as _,
                0,
                gl::RGBA as _,
                gl::UNSIGNED_BYTE,
                transmute(&(data[0]) as *const T)
            )),
            (TextureType::Texture2D, TextureFormat::FloatingPoint) => gl_function!(TexImage2D(
                self.1,
                0,
                gl::RGBA16F as _,
                width as _,
                height as _,
                0,
                gl::RGBA as _,
                gl::FLOAT,
                transmute(&(data[0]) as *const T)
            )),
            _ => unimplemented!(),
        }
    }

    pub fn allocate_space(&self, width: u32, height: u32) {
        self.allocate_space_with_format(width, height, TextureFormat::UnsignedByte);
    }

    pub fn allocate_space_with_format(&self, width: u32, height: u32, format: TextureFormat) {
        match (self.2, format) {
            (TextureType::Texture2D, TextureFormat::UnsignedByte) => gl_function!(TexImage2D(
                self.1,
                0,
                gl::RGB as _,
                width as _,
                height as _,
                0,
                gl::RGB as _,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            )),
            (TextureType::Texture2D, TextureFormat::UnsignedByteWithAlpha) => gl_function!(TexImage2D(
                self.1,
                0,
                gl::RGBA as _,
                width as _,
                height as _,
                0,
                gl::RGBA as _,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            )),
            (TextureType::Texture2D, TextureFormat::FloatingPoint) => gl_function!(TexImage2D(
                self.1,
                0,
                gl::RGBA16F as _,
                width as _,
                height as _,
                0,
                gl::RGBA as _,
                gl::FLOAT,
                ptr::null(),
            )),
            (TextureType::Texture2D, TextureFormat::Grey) => gl_function!(TexImage2D(
                self.1, 0, gl::RED as _, width as _, height as _, 0, gl::RED as _, gl::FLOAT, ptr::null(),
            )),
            _ => unimplemented!(),
        }
    }

    pub fn allocate_depth_space(&self, width: u32, height: u32) {
        match self.2 {
            TextureType::Texture2D => gl_function!(TexImage2D(
                self.1,
                0,
                gl::DEPTH_COMPONENT as _,
                width as _,
                height as _,
                0,
                gl::DEPTH_COMPONENT as _,
                gl::FLOAT,
                ptr::null(),
            )),
            _ => unimplemented!(),
        }
    }

    pub fn set_parameter(&self, parameter: gl::types::GLenum, value: gl::types::GLenum) {
        gl_function!(TexParameteri(self.1, parameter, value as i32));
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        gl_function!(DeleteTextures(1, &self.0));
    }
}
