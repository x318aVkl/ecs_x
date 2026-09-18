use std::rc::Rc;

use winit::{error::OsError, event_loop::{ActiveEventLoop, EventLoop}};



pub struct Window {
    window: Rc<winit::window::Window>,
}


impl Clone for Window {
    fn clone(&self) -> Self {
        Window {
            window: Rc::clone(&self.window),
        }
    }
}

pub struct PrimaryWindow;

impl Window {
    pub fn new(window: winit::window::Window) -> Self {
        Self {
            window: Rc::new(window),
        }
    }
    pub fn inner_ref(&self) -> &winit::window::Window {
        self.window.as_ref()
    }
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}




pub struct WindowManager {
    event_loop: *const ActiveEventLoop,
}


impl WindowManager {
    pub(super) fn attach(event_loop: &ActiveEventLoop) -> Self {
        Self {
            event_loop,
        }
    }
    fn event_loop(&self) -> &ActiveEventLoop {
        unsafe { &*self.event_loop }
    }

    pub fn create_window(&self, parameters: winit::window::WindowAttributes) -> Result<Window, OsError> {
        Ok(Window {
            window: Rc::new(
                self.event_loop().create_window(parameters)?
            )
        })
    }
}



