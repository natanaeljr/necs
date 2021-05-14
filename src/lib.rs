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

trait ComponentStorage {
    fn remove(&mut self, entity: &Entity);
    fn is_empty(&self) -> bool;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> ComponentStorage for HashMap<Entity, T> {
    fn remove(&mut self, entity: &Entity) {
        self.remove(entity);
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

///////////////////////////////////////////////////////////////////////////////

struct Registry {
    next: Entity,
    entities: HashMap<Entity, HashSet<ComponentId>>,
    component_pool: HashMap<ComponentId, Box<dyn ComponentStorage>>,
}

impl Registry {
    pub fn new() -> Self {
        Self { next: 1, entities: HashMap::new(), component_pool: HashMap::new() }
    }

    pub fn create(&mut self) -> Entity {
        let entity = self.next;
        self.next += 1;
        self.entities.insert(entity, HashSet::new());
        entity
    }

    pub fn destroy(&mut self, entity: Entity) {
        if let Some(component_ids) = self.entities.get(&entity) {
            for component_id in component_ids {
                let component_storage = self.component_pool.get_mut(component_id).unwrap().as_mut();
                component_storage.remove(&entity);
                if component_storage.is_empty() {
                    self.component_pool.remove(component_id);
                }
            }
        }
        self.entities.remove(&entity);
    }

    pub fn add<Component: Sized + 'static>(&mut self, entity: Entity, component: Component) {
        if let Some(component_ids) = self.entities.get_mut(&entity) {
            if component_ids.insert(TypeId::of::<Component>()) {
                match self.component_pool.entry(TypeId::of::<Component>()) {
                    Entry::Occupied(mut entry) => {
                        let map = entry.get_mut().as_any_mut().downcast_mut::<HashMap<Entity, Component>>().unwrap();
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
