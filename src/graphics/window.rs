use std::rc::Rc;

use winit::{error::OsError, event::MouseButton, event_loop::ActiveEventLoop};

use crate::ecs::ecs_table::Entity;



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
    pub fn id(&self) -> winit::window::WindowId {
        self.window.id()
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



// a pointer event
pub struct Pointer<T> {
    pub window: Entity,
    pub button: MouseButton,
    _event: T,
}


impl<T> Pointer<T> {
    pub fn new(window: Entity, button: MouseButton,  event: T) -> Self {
        Self { window, button, _event: event }
    }
}


#[derive(Debug)]
pub struct PointerPosition {
    pub x: f32,
    pub y: f32,
    pub(crate) initialized: bool,
}

pub struct Down;
pub struct Up;

pub struct PointerMoved {
    pub window: Entity,
    pub dx: f32,
    pub dy: f32,
}

