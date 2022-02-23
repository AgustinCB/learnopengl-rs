use itertools::Itertools;
use log::error;
use crate::render_buffer::RenderBuffer;
use crate::texture::{Texture, TextureFormat, TextureType};

#[derive(Debug)]
pub struct MultipleRenderTarget {
    _render_buffer: Option<RenderBuffer>,
    resource: gl::types::GLuint,
    pub textures: Vec<Texture>,
}

impl MultipleRenderTarget {
    pub fn new(width: u32, height: u32, targets: usize) -> MultipleRenderTarget {
        MultipleRenderTarget::new_with_format(width, height, targets, TextureFormat::UnsignedByte)
    }

    pub fn new_with_format(width: u32, height: u32, targets: usize, format: TextureFormat) -> MultipleRenderTarget {
        let mut frame_buffer = 0 as gl::types::GLuint;
        gl_function!(GenFramebuffers(1, &mut frame_buffer));
        gl_function!(BindFramebuffer(gl::FRAMEBUFFER, frame_buffer));

        let mut textures = vec![];
        for i in 0..targets {
            let texture = Texture::new(TextureType::Texture2D);
            texture.just_bind();
            texture.allocate_space_with_format(width, height, format);
            gl_function!(TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _));
            gl_function!(TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _));
            gl_function!(TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _));
            gl_function!(TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _));
            texture.unbind();
            gl_function!(FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0 + i as u32, TextureType::Texture2D as u32, texture.0, 0));
            textures.push(texture);
        }

        let render_buffer = RenderBuffer::new();
        render_buffer.bind();
        gl_function!(RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width as _, height as _));
        RenderBuffer::unbind();
        gl_function!(FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, render_buffer.0));

        let status = gl_function!(CheckFramebufferStatus(gl::FRAMEBUFFER));
        if status != gl::FRAMEBUFFER_COMPLETE {
            error!("Error creating frame buffer, status code {}", status);
        }
        MultipleRenderTarget::unbind();
        MultipleRenderTarget {
            textures,
            _render_buffer: Some(render_buffer),
            resource: frame_buffer,
        }
    }

    pub fn set_draw_buffers(&self) {
        let attachments = (0..self.textures.len()).into_iter()
            .map(|i| gl::COLOR_ATTACHMENT0 + i as u32)
            .collect_vec();
        gl_function!(DrawBuffers(self.textures.len() as _, attachments.as_ptr()));
    }

    pub fn bind(&self) {
        gl_function!(BindFramebuffer(gl::FRAMEBUFFER, self.resource));
    }

    pub fn unbind() {
        gl_function!(BindFramebuffer(gl::FRAMEBUFFER, 0));
    }
}

impl Drop for MultipleRenderTarget {
    fn drop(&mut self) {
        gl_function!(DeleteFramebuffers(1, &self.resource));
    }
}