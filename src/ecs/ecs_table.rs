use std::{any::{Any, TypeId}, cell::RefCell, collections::HashMap, hash::Hash};

use nohash_hasher::IsEnabled;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entity(u64);


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



pub(super) fn generate_entity(table: &EcsTable) -> Entity {
    let mut id = Entity(rand::random());

    while table_contains(table, id) {
        println!("collision!");
        id = Entity(rand::random());
    }
    
    return id;
}

fn table_contains(table: &EcsTable, id: Entity) -> bool {
    for (_, subtable) in table.iter() {
        if subtable.borrow().contains_key(&id) {
            return true;
        }
    }
    false
}