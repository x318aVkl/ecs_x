use std::{any::{Any, TypeId}, cell::RefCell};

use super::{ecs_table::{EcsTable, Entity, generate_entity}, scheduler::{AppStateInfo, CommandsQueue, Enter, Exit, Resources}, system::SystemParam};


#[derive(Debug)]
pub enum SystemMessage {
    Exit(u32),
    ChangeState{new_state: TypeId, new_enter_state: TypeId, new_exit_state: TypeId, data: Box<dyn Any>},
    WindowCreated(Entity),
}


pub struct Commands<'a> {
    queue: &'a CommandsQueue,
}

pub type StoredCommand = Box<dyn FnOnce(&mut Resources, &mut EcsTable) -> Option<SystemMessage>>;


pub trait Command {
    fn run(self, resources: &mut Resources);
}

impl<F> Command for F where F: FnOnce(&mut Resources) {
    fn run(self, resources: &mut Resources) {
        self(resources)
    }
}


impl<'a> SystemParam for Commands<'a> {
    type Item<'new> = Commands<'new>;
    fn retrieve<'r>(_resources: &'r Resources, _ecs_table: &'r EcsTable, commands: &'r CommandsQueue, _state: Option<AppStateInfo<'r>>) -> Self::Item<'r> {
        Commands {
            queue: commands,
        }
    }
}


impl<'a> Commands<'a> {

    // inserts the resource, overwrites any existing resource
    pub fn insert_resource<R: 'static>(&mut self, res: R) {
        self.queue.borrow_mut().push(Box::new(move |resources: &mut Resources, _table: &mut EcsTable| {
            resources.insert(TypeId::of::<R>(), RefCell::new(Box::new(res)));
            None
        }));
    }

    // inserts the resource only if it is not already there
    pub fn try_insert_resource<R: 'static>(&mut self, res: R) {
        self.queue.borrow_mut().push(Box::new(move |resources: &mut Resources, _table: &mut EcsTable| {
            resources.entry(TypeId::of::<R>()).or_insert(RefCell::new(Box::new(res)));
            None
        }));
    }

    pub fn exit(&mut self, code: u32) {
        self.queue.borrow_mut().push(Box::new(move |_resources: &mut Resources, _table: &mut EcsTable| {
            Some(SystemMessage::Exit(code))
        }));
    }

    #[allow(unused_variables)]
    pub fn change_state<S: 'static>(&mut self, state: S) {
        self.queue.borrow_mut().push(Box::new(move |_resources: &mut Resources, _table: &mut EcsTable| {
            Some(SystemMessage::ChangeState {
                new_state: TypeId::of::<S>(),
                new_enter_state: TypeId::of::<Enter<S>>(),
                new_exit_state: TypeId::of::<Exit<S>>(),
                data: Box::new(state),
            })
        }));
    }
}




pub trait SpawnEntity<Inputs> {
    fn spawn(&mut self, components: Inputs);
}


macro_rules! impl_commands_spawn_entity {
    (($($params:ident,)*), ($($n:tt,)*)) => {

        #[allow(unused_variables)]
        #[allow(non_snake_case)]
        impl<'a, $($params: 'static,)*> SpawnEntity<($($params,)*)> for Commands<'a> {

            fn spawn(&mut self, components: ($($params,)*)) {

                self.queue.borrow_mut().push(Box::new(move |_resources: &mut Resources, table: &mut EcsTable| {
                    let entity = generate_entity(table);

                    let mut window_created = false;
                    $(
                        let tid = TypeId::of::<$params>();
                        table.entry(tid).or_default().borrow_mut().insert(entity, RefCell::new(Box::new(components.$n)));

                        #[cfg(feature = "graphics")]
                        {
                            if tid == TypeId::of::<crate::graphics::window::Window>() {
                                window_created = true;
                            }
                        }
                    )*
                    
                    if window_created {
                        Some(SystemMessage::WindowCreated(entity))
                    } else {
                        None
                    }
                }));
            }
        }
        
    };
}


impl_commands_spawn_entity!((T0,), (0,));
impl_commands_spawn_entity!((T0, T1,), (0, 1,));
impl_commands_spawn_entity!((T0, T1, T2,), (0, 1, 2,));
impl_commands_spawn_entity!((T0, T1, T2, T3,), (0, 1, 2, 3,));
impl_commands_spawn_entity!((T0, T1, T2, T3, T4,), (0, 1, 2, 3, 4,));
impl_commands_spawn_entity!((T0, T1, T2, T3, T4, T5,), (0, 1, 2, 3, 4, 5,));
impl_commands_spawn_entity!((T0, T1, T2, T3, T4, T5, T6,), (0, 1, 2, 3, 4, 5, 6,));
impl_commands_spawn_entity!((T0, T1, T2, T3, T4, T5, T6, T7,), (0, 1, 2, 3, 4, 5, 6, 7,));
impl_commands_spawn_entity!((T0, T1, T2, T3, T4, T5, T6, T7, T8,), (0, 1, 2, 3, 4, 5, 6, 7, 8,));
impl_commands_spawn_entity!((T0, T1, T2, T3, T4, T5, T6, T7, T8, T9,), (0, 1, 2, 3, 4, 5, 6, 7, 8, 9,));
impl_commands_spawn_entity!((T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10,), (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,));
impl_commands_spawn_entity!((T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, ), (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,));
impl_commands_spawn_entity!((T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12,), (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,));



impl<'a> Commands<'a> {
    pub fn despawn(&mut self, entity: Entity) {
        self.queue.borrow_mut().push(Box::new(move |_res: &mut Resources, table: &mut EcsTable| {
            
            table.iter_mut().map(|(_, subtable)| {
                subtable.borrow_mut().remove(&entity);
            }).count();

            None
        }));
    }
}



