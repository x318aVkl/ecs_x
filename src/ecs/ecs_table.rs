use std::{any::{Any, TypeId}, cell::RefCell, collections::HashMap, hash::Hash};

use nohash_hasher::IsEnabled;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entity(pub u64);


pub type ComponentTable = HashMap<Entity, RefCell<Box<dyn Any>>, nohash_hasher::BuildNoHashHasher<Entity>>;
pub type ComponentTableIterator<'a> = std::collections::hash_map::Keys<'a, Entity, RefCell<Box<dyn Any>>>;

pub type EcsTable = HashMap<TypeId, RefCell<ComponentTable>>;



impl Hash for Entity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}


impl IsEnabled for Entity {
    
}