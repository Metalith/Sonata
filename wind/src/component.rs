use downcast_rs::Downcast;

use crate::entity::Entity;
use std::{
    any::{Any, TypeId},
    collections::HashSet,
};
use std::{cell::RefCell, collections::HashMap};

pub trait Component: Downcast + Any {}
impl<E: Any> Component for E {}
impl_downcast!(Component);

pub type Components = HashMap<TypeId, RefCell<Box<dyn Any>>>;

#[derive(Default)]
pub struct ComponentBuilder {
    components: Components,
}

impl ComponentBuilder {
    /// Creates an new builder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a component of type `C` to the entity.
    pub fn with<C: Component>(mut self, component: C) -> Self {
        self.components.insert(TypeId::of::<C>(), RefCell::new(Box::new(component)));
        self
    }

    /// Finishing the creation of the entity.
    pub fn build(self) -> Components {
        self.components
    }
}
#[derive(Default)]
pub struct ComponentStore {
    pub component_sets: HashMap<TypeId, HashSet<Entity>>,
    pub component_maps: HashMap<Entity, HashMap<TypeId, RefCell<Box<dyn Any>>>>,
}

impl ComponentStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_entity(&mut self, entity_id: Entity, components: Components) -> Result<(), &'static str> {
        if self.component_maps.contains_key(&entity_id) {
            return Err("Entity with this id already exists");
        }
        for (key, _) in &components {
            if !self.component_sets.contains_key(&key) {
                self.component_sets.insert(*key, HashSet::new());
            }
            self.component_sets.get_mut(&key).expect("Failed to add entity to component store").insert(entity_id);
        }
        self.component_maps.insert(entity_id, components);

        Ok(())
    }

    pub fn get_component<T: Any>(&self, entity_id: &Entity) -> Result<&T, &'static str> {
        match self.component_maps.get(&entity_id) {
            Some(component_map) => match component_map.get(&TypeId::of::<T>()) {
                Some(component) => Ok(component.as_any().downcast_ref::<T>().expect("ComponentStore: Internal downcast error")),
                _ => Err("Component not found"),
            },
            _ => Err("Entity not found"),
        }
    }

    pub fn get_component_mut<T: Any>(&mut self, entity_id: &Entity) -> Result<&mut T, &'static str> {
        match self.component_maps.get_mut(&entity_id) {
            Some(component_map) => match component_map.get_mut(&TypeId::of::<T>()) {
                Some(component) => Ok(component.get_mut().downcast_mut::<T>().expect("ComponentStore: Internal downcast error")),
                _ => Err("Component not found"),
            },
            _ => Err("Entity not found"),
        }
    }

    pub fn query_entities(&self, query: Vec<TypeId>) -> HashSet<Entity> {
        match query.first() {
            Some(first_type) => match self.component_sets.get(&first_type) {
                Some(intersect) => {
                    let mut res = intersect.clone();
                    for type_ in query.iter() {
                        if let Some(new_set) = self.component_sets.get(&type_) {
                            res = &res & new_set;
                        }
                    }
                    res
                }
                _ => HashSet::new(),
            },
            _ => HashSet::new(),
        }
    }
}
