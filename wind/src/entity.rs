use std::{
    any::{Any, TypeId},
    collections::HashSet,
};

use crate::component::{ComponentStore, Components};

pub type Entity = u32;

pub struct EntityBuilder<'a> {
    entity_components: Components,
    entity_store: &'a mut EntityStore,
    component_store: &'a mut ComponentStore,
}

impl<'a> EntityBuilder<'a> {
    pub fn new(entity_store: &'a mut EntityStore, component_store: &'a mut ComponentStore) -> Self {
        Self {
            entity_components: Components::default(),
            entity_store,
            component_store,
        }
    }

    pub fn components(mut self, components: Components) -> Self {
        self.entity_components = components;
        self
    }

    pub fn build(self) {
        let entity_id = self.entity_store.register_entity();
        self.component_store.register_entity(entity_id, self.entity_components);
    }
}

pub struct EntityFactory<'a> {
    entity_store: &'a mut EntityStore,
    component_store: &'a mut ComponentStore,
}

impl<'a> EntityFactory<'a> {
    pub fn new(entity_store: &'a mut EntityStore, component_store: &'a mut ComponentStore) -> Self {
        Self { entity_store, component_store }
    }

    pub fn create_entity(&mut self) -> EntityBuilder {
        EntityBuilder::new(&mut self.entity_store, &mut self.component_store)
    }

    pub fn query_entities(&self, query: EntityQuery) -> HashSet<Entity> {
        self.component_store.query_entities(query.0)
    }

    pub fn get_component<T: Any>(&mut self, entity_id: &Entity) -> Result<&T, &'static str> {
        self.component_store.get_component(&entity_id)
    }

    pub fn get_component_mut<T: Any>(&mut self, entity_id: &Entity) -> Result<&mut T, &'static str> {
        self.component_store.get_component_mut(&entity_id)
    }
}

#[derive(Default)]
pub struct EntityStore {
    entities: Vec<Entity>,
    entity_counter: u32,
}

impl EntityStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_entity(&mut self) -> Entity {
        let entity = self.entity_counter;
        self.entity_counter += 1;

        self.entities.push(entity);
        entity
    }
}

pub struct EntityQuery(pub Vec<TypeId>);

impl EntityQuery {
    pub fn new<T: Any>() -> Self {
        Self(vec![TypeId::of::<T>()])
    }

    pub fn and<T: Any>(mut self) -> Self {
        self.0.push(TypeId::of::<T>());
        self
    }
}
