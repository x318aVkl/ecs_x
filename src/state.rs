use std::{any::{Any, TypeId}, cell::{Ref, RefMut}, marker::PhantomData, ops::{Deref, DerefMut}};

use crate::system::SystemParam;



pub struct State<'a, T: 'static> {
    value: Ref<'a, Box<dyn Any>>,
    marker: PhantomData<&'a T>,
}

impl<'res, T: 'static> SystemParam for Option<State<'res, T>> {
    type Item<'new> = Option<State<'new, T>>;

    fn retrieve<'r>(_resources: &'r crate::scheduler::Resources, _ecs_table: &'r crate::ecs_table::EcsTable, _commands: &'r crate::scheduler::CommandsQueue, state: Option<crate::scheduler::AppStateInfo<'r>>) -> Self::Item<'r> {
        if let Some(state) = state {
            if state.current_state == TypeId::of::<T>() {
                Some(State {
                    value: state.current_data.borrow(),
                    marker: PhantomData
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'a, T> Deref for State<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value.downcast_ref().unwrap()
    }
}



pub struct StateMut<'a, T: 'static> {
    value: RefMut<'a, Box<dyn Any>>,
    marker: PhantomData<&'a T>,
}

impl<'res, T: 'static> SystemParam for Option<StateMut<'res, T>> {
    type Item<'new> = Option<StateMut<'new, T>>;

    fn retrieve<'r>(_resources: &'r crate::scheduler::Resources, _ecs_table: &'r crate::ecs_table::EcsTable, _commands: &'r crate::scheduler::CommandsQueue, state: Option<crate::scheduler::AppStateInfo<'r>>) -> Self::Item<'r> {
        if let Some(state) = state {
            if state.current_state == TypeId::of::<T>() {
                Some(StateMut {
                    value: state.current_data.borrow_mut(),
                    marker: PhantomData
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'a, T> Deref for StateMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value.downcast_ref().unwrap()
    }
}
impl<'a, T> DerefMut for StateMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.downcast_mut().unwrap()
    }
}

