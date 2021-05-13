#![allow(dead_code)]

use std::any::{Any, TypeId};
use std::collections::{HashSet, HashMap};
use std::collections::hash_map::Entry;

#[cfg(test)]
mod tests;

///////////////////////////////////////////////////////////////////////////////

type Entity = u64;

const NULL_ENTITY: Entity = 0;

///////////////////////////////////////////////////////////////////////////////

type ComponentId = TypeId;

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct Registry {
    next: Entity,
    entities: HashMap<Entity, HashSet<ComponentId>>,
    components: HashMap<ComponentId, Box<dyn Any>>,
}

impl Registry {
    pub fn new() -> Self {
        Self { next: 1, entities: HashMap::new(), components: HashMap::new() }
    }

    pub fn create(&mut self) -> Entity {
        let entity = self.next;
        self.next += 1;
        self.entities.insert(entity, HashSet::new());
        entity
    }

    pub fn destroy(&mut self, entity: Entity) {
        self.entities.remove(&entity);
    }

    pub fn add<Component: Sized + 'static>(&mut self, entity: Entity, component: Component) {
        if let Some(components) = self.entities.get_mut(&entity) {
            if components.insert(TypeId::of::<Component>()) {
                match self.components.entry(TypeId::of::<Component>()) {
                    Entry::Occupied(mut entry) => {
                        let map = entry.get_mut().downcast_mut::<HashMap<Entity, Component>>().unwrap();
                        map.insert(entity, component);
                    }
                    Entry::Vacant(entry) => {
                        let mut map: HashMap<Entity, Component> = HashMap::new();
                        map.insert(entity, component);
                        entry.insert(Box::new(map));
                    }
                }
            }
        }
    }

    pub fn remove<Component: Sized + 'static>(&mut self, entity: Entity) -> bool {
        self.entities.get_mut(&entity).and_then(|components| {
            components.remove(&TypeId::of::<Component>()).then(|| ())
        }).is_some()
    }

    pub fn replace<Component: Sized + 'static>(&mut self, entity: Entity, _component: Component) -> bool {
        self.entities.get_mut(&entity).and_then(|components| {
            components.contains(&TypeId::of::<Component>()).then(|| ())
        }).is_some()
    }
}

///////////////////////////////////////////////////////////////////////////////

struct Handle<'reg> {
    registry: &'reg mut Registry,
    entity: Entity,
}

impl<'reg> Handle<'reg> {
    fn new(registry: &'reg mut Registry, entity: Entity) -> Self {
        Self { registry, entity }
    }

    fn id(&self) -> Entity { self.entity }

    fn add<Component>(&mut self, _component: Component) {}

    fn remove<Component>(&mut self) {}

    fn replace<Component>(&mut self, _component: Component) {}
}
