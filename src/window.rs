use gl;
use sdl2::{EventPump, Sdl, TimerSubsystem, VideoSubsystem};
use sdl2::event::EventPollIterator;
use sdl2::mouse::MouseUtil;
use sdl2::video::{GLContext, GLProfile, Window as SDL2Window};

pub struct Window {
    events: Option<EventPump>,
    _gl_context: GLContext,
    sdl_context: Sdl,
    timer: TimerSubsystem,
    _video: VideoSubsystem,
    window: SDL2Window,
    now: usize,
    last: usize,
}

impl Window {
    pub fn new_with_anti_alias(name: &str, width: usize, height: usize, size: u8) -> Result<Window, String> {
        let sdl_context = sdl2::init()?;
        let video = sdl_context.video()?;
        let attrs = video.gl_attr();

        attrs.set_stencil_size(8);
        attrs.set_context_major_version(4);
        #[cfg(target_os = "macos")]
            attrs.set_context_minor_version(1);
        #[cfg(target_os = "linux")]
            attrs.set_context_minor_version(6);
        attrs.set_context_profile(GLProfile::Core);
        #[cfg(target_os = "macos")]
            attrs.set_context_flags().forward_compatible().set();
        attrs.set_multisample_buffers(1);
        attrs.set_multisample_samples(size);
        sdl_context.mouse().capture(true);
        sdl_context.mouse().set_relative_mouse_mode(true);

        let window = video
            .window(name, width as _, height as _)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;
        let gl_context = window.gl_create_context().unwrap();
        gl::load_with(|s| video.gl_get_proc_address(s) as *const std::os::raw::c_void);
        gl_function!(Enable(gl::MULTISAMPLE));
        let sdl_timer = sdl_context.timer().unwrap();
        Ok(Window {
            sdl_context,
            _video: video,
            window,
            events: None,
            last: 0,
            now: 0,
            timer: sdl_timer,
            _gl_context: gl_context,
        })
    }

    pub fn new(name: &str, width: usize, height: usize) -> Result<Window, String> {
        let sdl_context = sdl2::init()?;
        let video = sdl_context.video()?;
        let attrs = video.gl_attr();

        attrs.set_stencil_size(8);
        attrs.set_context_major_version(4);
        attrs.set_context_minor_version(1);
        attrs.set_context_profile(GLProfile::Core);
        #[cfg(target_os = "macos")]
            attrs.set_context_flags().forward_compatible().set();
        sdl_context.mouse().capture(true);
        sdl_context.mouse().set_relative_mouse_mode(true);

        let window = video
            .window(name, width as _, height as _)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;
        let gl_context = window.gl_create_context().unwrap();
        gl::load_with(|s| video.gl_get_proc_address(s) as *const std::os::raw::c_void);
        let sdl_timer = sdl_context.timer().unwrap();
        Ok(Window {
            sdl_context,
            _video: video,
            window,
            events: None,
            last: 0,
            now: 0,
            timer: sdl_timer,
            _gl_context: gl_context,
        })
    }

    pub fn start_timer(&mut self) {
        self.now = self.timer.performance_counter() as _;
    }

    pub fn delta_time(&mut self) -> f32 {
        self.last = self.now;
        self.now = self.timer.performance_counter() as _;
        ((self.now - self.last) / 1000) as f32
    }

    pub fn ticks(&self) -> u32 {
        self.timer.ticks()
    }

    pub fn swap_buffers(&self) {
        self.window.gl_swap_window()
    }

    pub fn delay(&mut self, ms: usize) {
        self.timer.delay(ms as _);
    }

    pub fn get_pumper(&mut self) -> EventPump {
        self.sdl_context.event_pump().unwrap()
    }

    pub fn events(&mut self) -> EventPollIterator {
        if self.events.is_none() {
            self.events = Some(self.get_pumper())
        }
        self.events.as_mut().unwrap().poll_iter()
    }

    pub fn mouse(&self) -> MouseUtil {
        self.sdl_context.mouse()
    }
}
