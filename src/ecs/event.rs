use std::{cell::RefCell, marker::PhantomData};

use super::{commands::Commands, plugin::Plugin, scheduler::{AddSystems, Startup, Update}, system::ResMut};



pub struct EventReader<T> {
    events: Vec<(T, RefCell<bool>)>,
}


pub struct EventWriter<T> {
    events: Vec<T>,
}


pub struct EventPlugin<Event> {
    _marker: PhantomData<Event>,
}

impl<T> Default for EventPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData
        }
    }
}


impl<Event: 'static> Plugin for EventPlugin<Event> {
    fn setup(self, app: super::scheduler::App) -> super::scheduler::App {
        app
            .add_systems::<Startup>((add_event_resources::<Event>,))
            .add_systems::<Update>((update_event_resources::<Event>,))
    }
}


fn add_event_resources<Event: 'static>(
    mut commands: Commands
) {
    commands.insert_resource(EventReader::<Event> {
        events: vec![]
    });
    commands.insert_resource(EventWriter::<Event> {
        events: vec![]
    });
}


fn update_event_resources<Event: 'static>(
    mut reader: ResMut<EventReader<Event>>,
    mut writer: ResMut<EventWriter<Event>>,
) {
    // clear the reader events that have been read
    reader.events.retain(|item| {
        *item.1.borrow() == false
    });

    let wlen = writer.events.len();
    for event in writer.events.drain(0..wlen) {
        reader.events.push((event, RefCell::new(false)));
    }
}

impl<Event> EventReader<Event> {
    pub fn read(&self) -> impl Iterator<Item = &Event> {
        self.events.iter().map(|(event, flag)| {
            *flag.borrow_mut() = true;
            event
        })
    }
}


impl<Event> EventWriter<Event> {
    pub fn write(&mut self, event: Event) {
        self.events.push(event);
    }
}





