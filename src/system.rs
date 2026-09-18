use std::{any::Any, cell::{Ref, RefMut}, marker::PhantomData, ops::{Deref, DerefMut}};

use crate::{ecs_table::EcsTable, scheduler::{AppStateInfo, CommandsQueue, Resources}};

use std::any::TypeId;




pub trait System {
    fn run(&mut self, resources: &Resources, ecs_table: &EcsTable, commands: &CommandsQueue, state: Option<AppStateInfo>);
}



pub type StoredSystem = Box<dyn System>;


pub struct FunctionSystem<Input, F> {
    f: F,
    marker: PhantomData<fn() -> Input>,
}


pub trait IntoSystem<Input> {
    type System: System;

    fn into_system(self) -> Self::System;
}




macro_rules! impl_system {
    ($($params:ident),*) => {
        #[allow(unused_variables)]
        #[allow(non_snake_case)]
        impl<F, $($params: SystemParam),*> System for FunctionSystem<($($params,)*), F> 
        where
            for<'a, 'b> &'a mut F:
            FnMut($($params),*) + 
            FnMut($(<$params as SystemParam>::Item<'b>),*)
        {
            fn run(&mut self, resources: &Resources, ecs_table: &EcsTable, commands: &CommandsQueue, state: Option<AppStateInfo>) {
                fn call_inner<$($params),*>(
                    mut f: impl FnMut($($params),*),
                    $($params: $params),*
                ) {
                    f($($params),*)
                }
                $(
                    let $params = <$params as SystemParam>::retrieve(resources, ecs_table, commands, state);
                )*

                call_inner(&mut self.f, $($params),*)
            }
        }


        impl<F: FnMut($($params),*), $($params: SystemParam),*> IntoSystem<($($params,)*)> for F 
        where
            for<'a, 'b> &'a mut F:
            FnMut($($params),*) + 
            FnMut($(<$params as SystemParam>::Item<'b>),*)
        {
            type System = FunctionSystem<($($params,)*), Self>;

            fn into_system(self) -> Self::System {
                FunctionSystem {
                    f: self,
                    marker: Default::default(),
                }
            }
        }
    }
}

impl_system!();
impl_system!(T1);
impl_system!(T1, T2);
impl_system!(T1, T2, T3);
impl_system!(T1, T2, T3, T4);


pub trait SystemParam {

    type Item<'new>;

    fn retrieve<'r>(resources: &'r Resources, ecs_table: &'r EcsTable, commands: &'r CommandsQueue, state: Option<AppStateInfo<'r>>) -> Self::Item<'r>;   
}

pub struct Res<'a, T: 'static> {
    value: Ref<'a, Box<dyn Any>>,
    marker: PhantomData<&'a T>,
}


impl<'a, T> Deref for Res<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value.downcast_ref().unwrap()
    }
}

impl<'res, T: 'static> SystemParam for Res<'res, T> {
    type Item<'new> = Res<'new, T>;

    fn retrieve<'r>(resources: &'r Resources, _ecs_table: &'r EcsTable, _commands: &'r CommandsQueue, _state: Option<AppStateInfo<'r>>) -> Self::Item<'r> {
        Res {
            value: match resources.get(&TypeId::of::<T>()) {
                Some(v) => v,
                None => panic!("Resource not found of type {:?}", TypeId::of::<T>())
            }.borrow(),
            marker: PhantomData,
        }
    }
}



pub struct ResMut<'a, T: 'static> {
    value: RefMut<'a, Box<dyn Any>>,
    marker: PhantomData<&'a T>,
}


impl<'a, T> Deref for ResMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value.downcast_ref().unwrap()
    }
}
impl<'a, T> DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.downcast_mut().unwrap()
    }
}

impl<'res, T: 'static> SystemParam for ResMut<'res, T> {
    type Item<'new> = ResMut<'new, T>;

    fn retrieve<'r>(resources: &'r Resources, _ecs_table: &'r EcsTable, _commands: &'r CommandsQueue, _state: Option<AppStateInfo>) -> Self::Item<'r> {
        ResMut {
            value: match resources.get(&TypeId::of::<T>()) {
                Some(v) => v,
                None => panic!("Resource not found of type {:?}", TypeId::of::<T>())
            }.borrow_mut(),
            marker: PhantomData,
        }
    }
}

