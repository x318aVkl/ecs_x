use std::{any::{Any, TypeId}, cell::{Ref, RefCell, RefMut}, marker::PhantomData};

use super::{ecs_table::{ComponentTableIterator, EcsTable, Entity}, scheduler::AppStateInfo, system::SystemParam};


pub struct Query<'a, Components, Filters = ()> {
    table: &'a EcsTable,
    _marker: PhantomData<(Components, Filters)>,
}




impl<'a, C, F> SystemParam for Query<'a, C, F> {
    type Item<'new> = Query<'new, C, F>;
    fn retrieve<'r>(_resources: &'r super::scheduler::Resources, ecs_table: &'r EcsTable, _commands: &'r super::scheduler::CommandsQueue, _state: Option<AppStateInfo<'r>>) -> Self::Item<'r> {
        Query {
            table: ecs_table,
            _marker: PhantomData,
        }
    }
}


pub trait QueryGet {
    type Output<'new> where Self: 'new;
    fn get<'r>(&'r self, entity: Entity) -> Option<Self::Output<'r>>;
}

pub trait QueryIter: Sized {
    fn iter<'r>(&'r self) -> QueryIterator<'r, Self, ComponentTableIterator<'r>>;
}

macro_rules! impl_query_get {

    (($($params:ident),*), ($($filters:ident),*)) => {
        #[allow(unused_variables)]
        #[allow(non_snake_case)]
        impl<'a, $($params: 'a + QueryArg),*, $($filters: 'a + QueryFilter),*> QueryGet for Query<'a, ($($params,)*), ($($filters,)*)> 
        where 
        $(<$params as QueryArg>::Inner: 'static,)*
        $(<$filters as QueryFilter>::Inner: 'static,)*
        {
            type Output<'new> = ($(<$params as QueryArg>::Output<'new>,)*) where Self: 'new;

            fn get<'r>(&'r self, entity: Entity) -> Option<Self::Output<'r>> {

                // check the filters
                $(
                    match self.table.get(&TypeId::of::<<$filters as QueryFilter>::Inner>()) {
                        Some(t) => {
                            if unsafe { t.try_borrow_unguarded().unwrap()}.contains_key(&entity) == <$filters as QueryFilter>::inclusive() {
                                // this filter is okay
                            } else {
                                return None;
                            }
                        },
                        None => {
                            return None;
                        }
                    }
                )*

                // check the data
                Some((
                $(    
                match self.table.get(&TypeId::of::<<$params as QueryArg>::Inner>()) {
                    Some(t) => {
                        if let Some(c) = unsafe { t.try_borrow_unguarded().unwrap()}.get(&entity) {
                            <$params as QueryArg>::from_refcell(c)
                        } else {
                            return None;
                        }
                    },
                    None => {
                        return None;
                    }
                },
                )*

                ))
            }
        }

        impl<'a, $($params: 'a + QueryArg),*, $($filters: 'a + QueryFilter),*> QueryIter for Query<'a, ($($params,)*), ($($filters,)*)> 
        where 
        $(<$params as QueryArg>::Inner: 'static,)*
        $(<$filters as QueryFilter>::Inner: 'static,)*
        {
            fn iter<'r>(&'r self) -> QueryIterator<'r, Self, ComponentTableIterator<'r>> {
                // find the minimal data table size
                let mut mintable = None;
                let mut mintablelen = usize::MAX;
                $(
                    match self.table.get(&TypeId::of::<<$params as QueryArg>::Inner>()) {
                        Some(t) => {
                            let ts = t.borrow().len();
                            if mintable.is_none() {
                                mintable = Some(TypeId::of::<<$params as QueryArg>::Inner>());
                                mintablelen = ts;
                            } else {
                                if ts < mintablelen {
                                    mintable = Some(TypeId::of::<<$params as QueryArg>::Inner>());
                                    mintablelen = ts;
                                }
                            }
                        },
                        None => {
                        }
                    }
                )*

                if mintablelen == usize::MAX {
                    // no table matches, means we should iterate over nothing
                    return QueryIterator {
                        query: self,
                        entity_iterator: None,
                    };
                }

                let mintable = mintable.unwrap();

                QueryIterator {
                    query: self,
                    entity_iterator: Some(unsafe{self.table.get(&mintable).unwrap().try_borrow_unguarded().unwrap()}.keys()),
                }

            }
        }
    }

}


impl_query_get!((T1), ());
impl_query_get!((T1, T2), ());
impl_query_get!((T1, T2, T3), ());
impl_query_get!((T1, T2, T3, T4), ());
impl_query_get!((T1, T2, T3, T4, T5), ());
impl_query_get!((T1, T2, T3, T4, T5, T6), ());

impl_query_get!((T1), (F1));
impl_query_get!((T1, T2), (F1));
impl_query_get!((T1, T2, T3), (F1));
impl_query_get!((T1, T2, T3, T4), (F1));
impl_query_get!((T1, T2, T3, T4, T5), (F1));
impl_query_get!((T1, T2, T3, T4, T5, T6), (F1));


impl_query_get!((T1), (F1, F2));
impl_query_get!((T1, T2), (F1, F2));
impl_query_get!((T1, T2, T3), (F1, F2));
impl_query_get!((T1, T2, T3, T4), (F1, F2));
impl_query_get!((T1, T2, T3, T4, T5), (F1, F2));
impl_query_get!((T1, T2, T3, T4, T5, T6), (F1, F2));


impl_query_get!((T1), (F1, F2, F3));
impl_query_get!((T1, T2), (F1, F2, F3));
impl_query_get!((T1, T2, T3), (F1, F2, F3));
impl_query_get!((T1, T2, T3, T4), (F1, F2, F3));
impl_query_get!((T1, T2, T3, T4, T5), (F1, F2, F3));
impl_query_get!((T1, T2, T3, T4, T5, T6), (F1, F2, F3));




pub trait QueryIterate {
    type Output<'new> where Self: 'new;
    fn iter<'r>(&'r self) ->  impl Iterator<Item = Self::Output<'r>>;
}

pub struct QueryIterator<'a, Query, I> {
    query: &'a Query,
    entity_iterator: Option<I>,
}

impl<'a, 'b, Query: QueryGet, I: Iterator<Item = &'b Entity>> Iterator for QueryIterator<'b, Query, I> where 'a: 'b {
    type Item = (Entity, <Query as QueryGet>::Output<'b>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entity_iterator) = self.entity_iterator.as_mut() {
            match entity_iterator.next() {
                Some(e) => {
                    match self.query.get(*e) {
                        Some(c) => {
                            Some((*e, c))
                        },
                        None => {
                            None
                        }
                    }
                },
                None => {
                    None
                }
            }
        } else {
            None
        }
    }
}





pub trait QueryArg {
    type Inner;
    type Output<'out> where Self::Inner: 'out;

    fn from_refcell<'r>(refcell: &'r RefCell<Box<dyn Any>>) -> Self::Output<'r>;
}

unsafe fn cast<B>(v: &Box<dyn Any>) -> &B {
    
    // SAFETY: we are sure the erased type is `T` (modulo variance)
    // because......

    let v = v.as_ref() as *const dyn Any;
    
    unsafe { &*(v as *const B) }
}


unsafe fn cast_mut<B>(v: &mut Box<dyn Any>) -> &mut B {
    
    // SAFETY: we are sure the erased type is `T` (modulo variance)
    // because......
    let v = v.as_mut() as *mut dyn Any;
    
    unsafe { &mut *(v as *mut B) }
}



impl<T> QueryArg for &T {
    type Inner = T;
    type Output<'out> = Ref<'out, T> where T: 'out;

    fn from_refcell<'r>(refcell: &'r RefCell<Box<dyn Any>>) -> Self::Output<'r> {
        unsafe {
            Ref::map(refcell.borrow(), |x| cast::<T>(x))
        }
    }
}

impl<T> QueryArg for &mut T {
    type Inner = T;
    type Output<'out> = RefMut<'out, T> where T: 'out;

    fn from_refcell<'r>(refcell: &'r RefCell<Box<dyn Any>>) -> Self::Output<'r> {
        unsafe {
            RefMut::map(refcell.borrow_mut(), |x| cast_mut::<T>(x))
        }
    }
}



pub trait QueryFilter {
    type Inner;
    fn inclusive() -> bool;
}

pub struct With<T> {
    _marker: PhantomData<T>,
}
pub struct Without<T> {
    _marker: PhantomData<T>,
}

impl<T> QueryFilter for With<T> {
    type Inner = T;
    fn inclusive() -> bool {
        true
    }
}
impl<T> QueryFilter for Without<T> {
    type Inner = T;
    fn inclusive() -> bool {
        false
    }
}

