use downcast_rs::Downcast;

use crate::{component::Component, entity::EntityBuilder, entity::EntityFactory, World};

use std::any::{Any, TypeId};
use std::collections::HashMap;

pub trait System<E> {
    fn parse_event<'a>(&mut self, entity_factory: &'a mut EntityFactory<'a>, event: &E);
}

pub struct SystemBuilder<'a, E> {
    system: Box<dyn System<E>>,
    system_components: Vec<TypeId>,
    system_store: &'a mut SystemStore<E>,
}

impl<'a, E> SystemBuilder<'a, E> {
    pub fn new<S: System<E>>(system_store: &'a mut SystemStore<E>, system: S) -> Self
    where
        S: System<E> + 'static,
    {
        Self {
            system: Box::new(system),
            system_components: Vec::new(),
            system_store,
        }
    }

    pub fn with_component<C: Component>(mut self) -> Self {
        self.system_components.push(TypeId::of::<C>());
        self
    }

    pub fn build(self) {
        self.system_store.register(self.system, self.system_components);
    }
}

#[derive(Default)]
pub struct SystemStore<E> {
    pub systems: HashMap<u32, Box<dyn System<E>>>,
    pub system_components: HashMap<u32, Vec<TypeId>>,
    system_counter: u32,
}

impl<E> SystemStore<E> {
    pub fn new() -> SystemStore<E> {
        SystemStore {
            systems: HashMap::new(),
            system_components: HashMap::new(),
            system_counter: 0,
        }
    }

    pub fn register(&mut self, system: Box<dyn System<E>>, components: Vec<TypeId>) {
        let system_id = self.system_counter;
        self.system_counter += 1;

        self.systems.insert(system_id, system);
        self.system_components.insert(system_id, components);
    }
}
