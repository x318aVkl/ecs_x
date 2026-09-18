use std::{any::{Any, TypeId}, cell::RefCell, collections::HashMap};


use super::{commands::{StoredCommand, SystemMessage}, ecs_table::EcsTable, system::{IntoSystem, StoredSystem}, plugin::Plugin};

pub type Resources = HashMap<TypeId, RefCell<Box<dyn Any>>>;
pub type CommandsQueue = RefCell<Vec<StoredCommand>>;

pub type AppState = TypeId;




pub struct AppSettings {
    pub target_framerate: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            target_framerate: 240.,
        }
    }
}


pub struct Scheduler {
    systems: HashMap<TypeId, Vec<StoredSystem>>,
    pub(crate) resources: Resources,
    pub(crate) ecs_table: EcsTable,
    current_state: Option<AppState>,
    current_exit_state: Option<AppState>,
    current_state_data: Option<RefCell<Box<dyn Any>>>,
    pub(crate) settings: AppSettings,
}

pub type App = Scheduler;


pub struct SystemStartup;

pub struct Startup;
pub struct MainLoop;
pub struct Update;
pub struct Enter<T>(pub T);
pub struct Exit<T>(pub T);




#[derive(Clone, Copy)]
pub struct AppStateInfo<'a> {
    pub current_state: AppState,
    pub current_data: &'a RefCell<Box<dyn Any>>,
}



impl Scheduler {

    pub fn new() -> Self {
        Self { 
            systems: HashMap::new(),
            resources: HashMap::new(),
            ecs_table: HashMap::new(),
            current_state: None,
            current_exit_state: None,
            current_state_data: None,
            settings: Default::default(),
        }
    }

    #[allow(unused_variables)]
    fn run_stage<S: 'static>(&mut self, schedule: S) -> Result<(), SystemMessage> {
        self.run_stage_dynamic(TypeId::of::<S>())
    }


    #[allow(unused_variables)]
    fn run_stage_dynamic(&mut self, schedule: TypeId) -> Result<(), SystemMessage> {

        let mut queue_state_change = vec![];

        let state = if let Some(state) = self.current_state {
            Some(AppStateInfo {
                current_state: state,
                current_data: &self.current_state_data.as_ref().unwrap(),
            })
        } else {
            None
        };

        if let Some(systems) = self.systems.get_mut(&schedule) {
            for system in systems {
                let commands = RefCell::new(vec![]);
                system.run(&self.resources, &self.ecs_table, &commands, state);

                for command in commands.take() {
                    let msg = (command)(&mut self.resources, &mut self.ecs_table);
                    if let Some(msg) = msg {
                        match msg {
                            SystemMessage::Exit(code) => {
                                return Err(msg);
                            },
                            SystemMessage::ChangeState { new_state, new_enter_state, new_exit_state, data } => {
                                queue_state_change.push((new_state, new_enter_state, new_exit_state, data));
                            }
                        }
                    }
                }
            }
        }
        for (new_state, new_enter_state, new_exit_state, data) in queue_state_change {
            self.change_state(new_state, new_enter_state, new_exit_state, data)?;
        }
        Ok(())
    }


    fn change_state(&mut self, new_state: TypeId, new_enter_state: TypeId, new_exit_state: TypeId, new_data: Box<dyn Any>) -> Result<(), SystemMessage> {
        if let Some(s) = self.current_exit_state {
            self.run_stage_dynamic(s)?;
        }

        // chagne the stage
        self.current_state = Some(new_state);
        self.current_exit_state = Some(new_exit_state);
        self.current_state_data = Some(RefCell::new(new_data));

        self.run_stage_dynamic(new_enter_state)?;

        Ok(())
    }

    fn static_change_state<S: 'static>(&mut self, state: S) -> Result<(), SystemMessage> {
        self.change_state(TypeId::of::<S>(), TypeId::of::<Enter<S>>(), TypeId::of::<Exit<S>>(), Box::new(state))
    }

    pub fn run(mut self) -> Result<(), u32> {

        #[cfg(feature = "graphics")]
        {
            // create the event loop
            let event_loop = match winit::event_loop::EventLoop::new() {
                Ok(e) => e,
                Err(_) => {
                    return Err(1)
                }
            };

            match event_loop.run_app(&mut self) {
                Ok(()) => (),
                Err(_) => {
                    return Err(1);
                }
            }

            return Ok(());
        }

        #[allow(unreachable_code)]
        runner(self)
    }



    pub(crate) fn run_before_main_loop(&mut self) -> Result<(), SystemMessage> {
        // run the system startup stage, runs before the user startup
        if let Err(msg) = self.static_change_state(SystemStartup) {
            match msg {
                SystemMessage::Exit(_code) => {
                    return Err(msg);
                },
                _ => {}
            }
        }
        if let Err(msg) = self.run_stage(SystemStartup) {
            match msg {
                SystemMessage::Exit(_code) => {
                    return Err(msg);
                },
                _ => {}
            }
        }
        // start by entering the startup stage, this will run systems that should be ran before startup
        if let Err(msg) = self.static_change_state(Startup) {
            match msg {
                SystemMessage::Exit(_code) => {
                    return Err(msg);
                },
                _ => {}
            }
        }
        // run the startup systems
        if let Err(msg) = self.run_stage(Startup) {
            match msg {
                SystemMessage::Exit(_code) => {
                    return Err(msg);
                },
                _ => {}
            }
        }
        // enter the mainloop stage, this will run systems that should be ran after startup and before the main loop
        if let Err(msg) = self.static_change_state(MainLoop) {
            match msg {
                SystemMessage::Exit(_code) => {
                    return Err(msg);
                },
                _ => {}
            }
        }

        Ok(())
    }


    pub(crate) fn on_main_loop_update(&mut self) -> Result<(), SystemMessage> {
        if let Err(msg) = self.run_stage(Update) {
            match msg {
                SystemMessage::Exit(_code) => {
                    return Err(msg);
                },
                _ => {}
            }
        }
        Ok(())
    }
}



// the default runner when graphics is not enabled
pub fn runner(mut app: Scheduler) -> Result<(), u32> {

    match app.run_before_main_loop() {
        Ok(()) => (),
        Err(e) => {
            match e {
                SystemMessage::Exit(code) => {
                    if code == 0 {
                        return Ok(());
                    } else {
                        return Err(code);
                    }
                },
                _ => return Err(1)
            }
        }
    };
    
    if app.systems.contains_key(&TypeId::of::<Update>()) {
        loop {
            match app.on_main_loop_update() {
                Ok(()) => (),
                Err(e) => {
                    match e {
                        SystemMessage::Exit(code) => {
                            if code == 0 {
                                return Ok(());
                            } else {
                                return Err(code);
                            }
                        },
                        _ => return Err(1)
                    }
                }
            };
        }
    }

    Ok(())
}




pub trait AddSystems<Systems, Inputs> {
    fn add_systems<S: 'static>(self, systems: Systems) -> Self;
}


macro_rules! impl_add_systems {

    (($($systems:ident,)*), ($($inputs:ident,)*), ($($n:tt,)*)) => {
        impl<$($inputs,)* $($systems: IntoSystem<$inputs>,)*> AddSystems<($($systems,)*), ($($inputs,)*)> for Scheduler where $(<$systems as IntoSystem<$inputs>>::System: 'static,)* {
            fn add_systems<S: 'static>(mut self, systems: ($($systems,)*)) -> Self {
                let s_id = TypeId::of::<S>();

                let entry = self.systems.entry(s_id).or_default();

                $(
                    entry.push(Box::new(systems.$n.into_system()));
                )*

                self
            }
        }
    }
}


impl_add_systems!((I0,), (S0,), (0,));
impl_add_systems!((I0, I1,), (S0, S1,), (0, 1,));
impl_add_systems!((I0, I1, I2,), (S0, S1, S2,), (0, 1, 2,));
impl_add_systems!((I0, I1, I2, I3,), (S0, S1, S2, S3,), (0, 1, 2, 3,));



pub trait AddPlugins<Plugins> {
    fn add_plugins(self, plugins: Plugins) -> Self;
}

macro_rules! impl_add_plugins {

    (($($plugins:ident,)*), ($($n:tt,)*)) => {
        impl<$($plugins: Plugin,)*> AddPlugins<($($plugins,)*)> for Scheduler {
            fn add_plugins(mut self, plugins: ($($plugins,)*)) -> Self {

                $(
                    self = plugins.$n.setup(self);
                )*

                self
            }
        }
    }
}


impl_add_plugins!((T0,), (0,));
impl_add_plugins!((T0, T1,), (0, 1,));
impl_add_plugins!((T0, T1, T2,), (0, 1, 2,));
impl_add_plugins!((T0, T1, T2, T3,), (0, 1, 2, 3,));





