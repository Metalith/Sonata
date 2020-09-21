#[macro_use]
extern crate downcast_rs;
extern crate wind_codegen;

mod component;
mod entity;
mod system;

use component::ComponentStore;
use entity::{Entity, EntityStore};
use std::{any::Any, collections::HashSet};
use system::{SystemBuilder, SystemStore};

pub use component::ComponentBuilder;
pub use entity::{EntityBuilder, EntityFactory, EntityQuery};
pub use system::System;
pub use wind_codegen::*;

pub struct World<E> {
    system_store: SystemStore<E>,
    component_store: ComponentStore,
    entity_store: EntityStore,

    update_event: E,
}

impl<E> World<E> {
    pub fn new(update_event: E) -> World<E> {
        World {
            system_store: SystemStore::new(),
            component_store: ComponentStore::new(),
            entity_store: EntityStore::new(),
            update_event: update_event,
        }
    }

    pub fn create_system<T: System<E> + 'static>(&mut self, system: T) -> SystemBuilder<E> {
        SystemBuilder::new(&mut self.system_store, system)
    }

    pub fn create_entity(&mut self) -> EntityBuilder {
        EntityBuilder::new(&mut self.entity_store, &mut self.component_store)
    }

    pub fn get_component<T: Any>(&mut self, entity_id: &Entity) -> Result<&mut T, &'static str> {
        self.component_store.get_component_mut(entity_id)
    }

    pub fn update(&mut self) {
        for (_, system) in self.system_store.systems.iter_mut() {
            let mut factory = EntityFactory::new(&mut self.entity_store, &mut self.component_store);
            system.parse_event(&mut factory, &self.update_event);
        }
    }

    pub fn send_event(&mut self, event: &E) {
        for (_, system) in self.system_store.systems.iter_mut() {
            let mut factory = EntityFactory::new(&mut self.entity_store, &mut self.component_store);
            system.parse_event(&mut factory, &event);
        }
    }

    pub fn query_entities(&self, query: EntityQuery) -> HashSet<Entity> {
        self.component_store.query_entities(query.0)
    }
}
