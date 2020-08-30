use std::any::{Any, TypeId};

use std::collections::HashMap;

type Entity = u32;

pub struct EntityBuilder<'a> {
    entity_id: Entity,
    entity_store: &'a mut EntityStore,
    component_store: &'a mut ComponentStore,
}

impl<'a> EntityBuilder<'a> {
    fn new(entity_id: Entity, entity_store: &'a mut EntityStore, component_store: &'a mut ComponentStore) -> Self {
        Self {
            entity_id,
            entity_store,
            component_store,
        }
    }

    pub fn components(self, components: Components) {
        self.component_store.register_entity(self.entity_id, components)
    }

    pub fn build(self) -> Entity {
        self.entity_store.register_entity(self.entity_id);
        self.entity_id
    }
}

pub trait System: Any {
    fn update(&self, entities: &HashMap<TypeId, Vec<Entity>>);
}
pub trait Component: Any {}
impl<E: Any> Component for E {}

type Components = HashMap<TypeId, Box<dyn Component>>;

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
        self.components.insert(TypeId::of::<C>(), Box::new(component));
        self
    }

    /// Finishing the creation of the entity.
    pub fn build(self) -> Components {
        self.components
    }
}

#[derive(Default)]
pub struct EntityStore {
    entities: Vec<Entity>,
}

impl EntityStore {
    fn new() -> Self {
        Self::default()
    }

    pub fn register_entity(&mut self, entity_id: Entity) {
        self.entities.push(entity_id);
    }
}

#[derive(Default)]
pub struct ComponentStore {
    component_map: HashMap<TypeId, HashMap<Entity, Box<dyn Component>>>,
}

impl ComponentStore {
    fn new() -> Self {
        Self::default()
    }

    fn register_entity(&mut self, entity_id: Entity, components: Components) {
        for (key, value) in components {
            if !self.component_map.contains_key(&key) {
                self.component_map.insert(key, HashMap::default());
            }

            self.component_map.get_mut(&key).expect("Failed to add entity to component store").insert(entity_id, value);
        }
    }
}

pub struct SystemBuilder<'a> {
    system_id: u32,
    system: Box<dyn System>,
    system_store: &'a mut SystemStore,
}

impl<'a> SystemBuilder<'a> {
    fn new<S: System>(system_store: &'a mut SystemStore, system: S, system_id: u32) -> Self {
        Self {
            system_id,
            system: Box::new(system),
            system_store,
        }
    }

    pub fn with_component<C: Component>(self) -> Self {
        self.system_store.register_component(self.system_id, TypeId::of::<C>());
        self
    }

    pub fn build(self) -> u32 {
        self.system_store.register(self.system_id, self.system);
        self.system_id
    }
}

#[derive(Default)]
pub struct SystemStore {
    pub systems: HashMap<u32, Box<dyn System>>,
    pub system_components: HashMap<u32, Vec<TypeId>>,
}

impl SystemStore {
    fn new() -> Self {
        Self::default()
    }

    fn register(&mut self, system_id: u32, system: Box<dyn System>) {
        self.systems.insert(system_id, system);
        self.system_components.insert(system_id, Vec::new());
    }

    fn register_component(&mut self, system_id: u32, component: TypeId) {
        self.system_components.get_mut(&system_id).expect("failed to register system").push(component);
    }
}

pub struct Engine {
    system_store: SystemStore,
    component_store: ComponentStore,
    entity_store: EntityStore,

    system_counter: u32,
    entity_counter: u32,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            system_store: SystemStore::new(),
            component_store: ComponentStore::new(),
            entity_store: EntityStore::new(),
            system_counter: 0,
            entity_counter: 0,
        }
    }

    pub fn create_system<T: System>(&mut self, system: T) -> SystemBuilder {
        let system_id = self.system_counter.into();
        self.system_counter += 1;

        SystemBuilder::new(&mut self.system_store, system, system_id)
    }

    pub fn create_entity(&mut self) -> EntityBuilder {
        let entity = self.entity_counter.into();
        self.entity_counter += 1;

        EntityBuilder::new(entity, &mut self.entity_store, &mut self.component_store)
    }

    pub fn update(&self) {
        for (system_id, system) in self.system_store.systems.iter() {
            let mut entities = HashMap::new();
            match self.system_store.system_components.get(&system_id) {
                Some(components) => {
                    for component in components.iter() {
                        match self.component_store.component_map.get(component) {
                            Some(entity_components) => {
                                entities.insert(*component, entity_components.keys().cloned().collect::<Vec<Entity>>());
                            }
                            _ => (),
                        };
                    }
                }
                _ => (),
            };
            // for components in self.system_store.system_components.get(sys.0).expect("system h");
            system.update(&entities);
        }
    }
}
