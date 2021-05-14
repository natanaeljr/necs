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

    pub fn add<Component: Sized + 'static>(&mut self, entity: Entity, new_component: Component) {
        if let Some(component_ids) = self.entities.get_mut(&entity) {
            if component_ids.insert(TypeId::of::<Component>()) {
                match self.component_pool.entry(TypeId::of::<Component>()) {
                    Entry::Occupied(mut entry) => {
                        let map = entry.get_mut().as_any_mut().downcast_mut::<HashMap<Entity, Component>>().unwrap();
                        map.insert(entity, new_component);
                    }
                    Entry::Vacant(entry) => {
                        let mut map: HashMap<Entity, Component> = HashMap::new();
                        map.insert(entity, new_component);
                        entry.insert(Box::new(map));
                    }
                }
            }
        }
    }

    pub fn remove<Component: Sized + 'static>(&mut self, entity: Entity) {
        if let Some(component_ids) = self.entities.get_mut(&entity) {
            if component_ids.remove(&TypeId::of::<Component>()) {
                let component_storage = self.component_pool.get_mut(&TypeId::of::<Component>()).unwrap().as_mut();
                component_storage.remove(&entity);
                if component_storage.is_empty() {
                    self.component_pool.remove(&TypeId::of::<Component>());
                }
            }
        }
    }

    pub fn replace<Component: Sized + 'static>(&mut self, entity: Entity, new_component: Component) {
        match self.component_pool.entry(TypeId::of::<Component>()) {
            Entry::Occupied(mut entry) => {
                let component_storage = entry.get_mut().as_any_mut().downcast_mut::<HashMap<Entity, Component>>().unwrap();
                if let Some(old_component) = component_storage.get_mut(&entity) {
                    *old_component = new_component;
                }
            }
            Entry::Vacant(_) => {}
        }
    }

    pub fn get<Component: Sized + 'static>(&self, entity: Entity) -> Option<&Component> {
        if let Some(component_pool) = self.component_pool.get(&TypeId::of::<Component>()) {
            let component_pool = component_pool.as_ref();
            let component_storage = component_pool.as_any().downcast_ref::<HashMap<Entity, Component>>().unwrap();
            return component_storage.get(&entity)
        }
        None
    }

    pub fn exists(&self, entity: Entity) -> bool {
        self.entities.contains_key(&entity)
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

    #[inline]
    fn id(&self) -> Entity { self.entity }

    #[inline]
    fn add<Component: Sized + 'static>(&mut self, new_component: Component) {
        self.registry.add(self.entity, new_component)
    }

    #[inline]
    fn remove<Component: Sized + 'static>(&mut self) {
        self.registry.remove::<Component>(self.entity)
    }

    #[inline]
    fn replace<Component: Sized + 'static>(&mut self, new_component: Component) {
        self.registry.replace(self.entity, new_component)
    }

    #[inline]
    fn get<Component: Sized + 'static>(&mut self) -> Option<&Component>{
        self.registry.get::<Component>(self.entity)
    }
}
