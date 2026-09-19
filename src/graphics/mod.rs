use std::{any::TypeId, cell::{Ref, RefCell, RefMut}};

use winit::{application::ApplicationHandler, event::ElementState};

use crate::{ecs::{commands::{Commands, SpawnEntity, SystemMessage}, event::{EventPlugin, EventWriter}, plugin::Plugin, query::{Query, QueryIter, With}, scheduler::{AddPlugins, AddSystems, Scheduler, SystemStartup, Update}, system::Res}, graphics::window::{Down, Pointer, PointerMoved, PointerPosition, PrimaryWindow, Up, WindowManager}};



pub mod window;
pub mod render;



pub use window::Window;


// the render stage
pub struct Render;


impl ApplicationHandler for Scheduler {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {

        // create and add the window manager
        let window_manager = WindowManager::attach(event_loop);
        self.resources.insert(TypeId::of::<WindowManager>(), RefCell::new(Box::new(window_manager)));


        match self.run_before_main_loop() {
            Err(SystemMessage::Exit(code)) => {
                if code != 0 {
                    println!("Warning, exit code non-zero: {}", code);
                }
                event_loop.exit();
            },
            _ => {}
        }
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    )
    {
        use winit::event::WindowEvent;

        // get the down pointer event writer
        let window: crate::ecs::ecs_table::Entity = *self.window_id_to_entity.get(&window_id.into()).unwrap();

        match event {
            WindowEvent::MouseInput { device_id: _, state, button } => {
                match state {
                    ElementState::Pressed => {
                        let down_writer = self.resources.get(&TypeId::of::<EventWriter<Pointer<Down>>>()).unwrap();
                        let mut down_writer = RefMut::map(down_writer.borrow_mut(), |b| b.downcast_mut::<EventWriter<Pointer<Down>>>().unwrap());
                        down_writer.write(Pointer::<Down>::new(window, button, Down));
                    },
                    ElementState::Released => {
                        let up_writer = self.resources.get(&TypeId::of::<EventWriter<Pointer<Up>>>()).unwrap();
                        let mut up_writer = RefMut::map(up_writer.borrow_mut(), |b| b.downcast_mut::<EventWriter<Pointer<Up>>>().unwrap());
                        up_writer.write(Pointer::<Up>::new(window, button, Up));
                    },
                }
            },
            WindowEvent::CursorMoved { device_id: _, position } => {
                // attempt to get the cursor position resource to get the last cursor position
                if let Some(cursor_pos) = self.resources.get(&TypeId::of::<PointerPosition>()) {

                    {
                        let cursor_pos = Ref::map(cursor_pos.borrow(), |f| f.downcast_ref::<PointerPosition>().unwrap());
                        println!("cursor pos: {:?}", cursor_pos);
                        if cursor_pos.initialized {
                            let movement_event = PointerMoved {
                                window,
                                dx: position.x as f32 - cursor_pos.x,
                                dy: position.y as f32 - cursor_pos.y,
                            };

                            let moved_writer = self.resources.get(&TypeId::of::<EventWriter<PointerMoved>>()).unwrap();
                            let mut moved_writer = RefMut::map(moved_writer.borrow_mut(), |b| b.downcast_mut::<EventWriter<PointerMoved>>().unwrap());
                            moved_writer.write(movement_event);
                        }
                    }

                    // change the pointer position resource
                    self.resources.insert(TypeId::of::<PointerPosition>(), RefCell::new(Box::new(PointerPosition {x: position.x as f32, y: position.y as f32, initialized: true})));

                } else {
                    // spawn the cursor position resource, do not send a cursor moved event
                    self.resources.insert(TypeId::of::<PointerPosition>(), RefCell::new(Box::new(PointerPosition {x: position.x as f32, y: position.y as f32, initialized: true})));
                }
            },
            WindowEvent::CloseRequested => {
                println!("destroyed window");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                // we have to draw things here
                match self.run_stage(Render) {
                    Err(SystemMessage::Exit(code)) => {
                        if code != 0 {
                            println!("Warning, exit code non-zero: {}", code);
                        }
                        event_loop.exit();
                    },
                    _ => {}
                }
            },
            _ => {}
        };
    }


    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {

        // update the window manager to ensure the reference is valid
        let window_manager = WindowManager::attach(event_loop);
        self.resources.insert(TypeId::of::<WindowManager>(), RefCell::new(Box::new(window_manager)));

        // handle the logic
        match self.on_main_loop_update() {
            Err(SystemMessage::Exit(code)) => {
                if code != 0 {
                    println!("Warning, exit code non-zero: {}", code);
                }
                event_loop.exit();
            },
            _ => {}
        }

        // now request a refresh at a rate of 60 fps
        let now = std::time::Instant::now();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            now + std::time::Duration::from_secs_f32(1.0 / self.settings.target_framerate)
        ));
    }
}





pub struct WindowPlugin {
    pub primary_window: winit::window::WindowAttributes,
    pub exit_on_primary_window_close: bool,
}

impl Default for WindowPlugin {
    fn default() -> Self {
        Self {
            primary_window: Default::default(),
            exit_on_primary_window_close: true,
        }
    }
}


impl Plugin for WindowPlugin {
    fn setup(self, app: crate::ecs::scheduler::App) -> crate::ecs::scheduler::App {
        app
            .add_plugins((
                EventPlugin::<Pointer<Down>>::default(),
                EventPlugin::<Pointer<Up>>::default(),
                EventPlugin::<PointerMoved>::default(),
            ))
            .add_systems::<SystemStartup>((create_primary_window_factory(self.primary_window),))
            .add_systems::<SystemStartup>((create_resources,))
            .add_systems::<Update>((create_window_autoexit_factory(self.exit_on_primary_window_close),))
            .add_systems::<Render>((render::render,))
    }
}


fn create_primary_window_factory(
    parameters: winit::window::WindowAttributes,
) -> impl Fn(
    Res<WindowManager>,
    Commands,
) {
    // spawn the primary window
    move |
        window_manager: Res<WindowManager>,
        mut commands: Commands,
    | {
        commands.spawn((
            match window_manager.create_window(parameters.clone()) {
                Ok(window) => {
                    window.request_redraw();
                    window
                },
                Err(e) => {
                    println!("Error, failed to create window, error: {:?}", e);
                    commands.exit(1);
                    return;
                }
            },
            PrimaryWindow,
        ));
    }
}

fn create_window_autoexit_factory(
    exit_on_window_destruct: bool
) -> impl Fn(Query<(&Window,), (With<PrimaryWindow>,)>, Commands) {
    move |query: Query<(&Window,), (With<PrimaryWindow>,)>, mut commands: Commands| {
        if exit_on_window_destruct && (query.iter().count() == 0) {
            commands.exit(0);
        }
    }
}

fn create_resources(
    mut commands: Commands,
) {
    commands.insert_resource(PointerPosition {
        x: 0.,
        y: 0.,
        initialized: false,
    });
}

