#![allow(dead_code)]

use std::any::{Any, TypeId};
use std::collections::{HashSet, HashMap};
use std::collections::hash_map::Entry;

#[cfg(test)]
mod tests;

pub type Entity = u64;

pub type ComponentId = TypeId;

pub trait ComponentTrait: 'static + Sized {}

impl<T: 'static + Sized> ComponentTrait for T {}

trait ComponentStorage {
    fn remove(&mut self, entity: &Entity);
    fn is_empty(&self) -> bool;
    fn len(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: ComponentTrait> ComponentStorage for HashMap<Entity, T> {
    fn remove(&mut self, entity: &Entity) { self.remove(entity); }
    fn is_empty(&self) -> bool { self.is_empty() }
    fn len(&self) -> usize { self.len() }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug, Default)]
struct Observer;

pub struct Registry {
    next: Entity,
    entities: HashMap<Entity, HashSet<ComponentId>>,
    component_pool: HashMap<ComponentId, Box<dyn ComponentStorage>>,
    observer: Observer,
}

impl Registry {
    pub fn new() -> Self {
        Self { next: 1, entities: HashMap::new(), component_pool: HashMap::new(), observer: Default::default() }
    }

    pub fn create(&mut self) -> Entity {
        let entity = self.next;
        self.next += 1;
        self.entities.insert(entity, HashSet::new());
        entity
    }

    pub fn create_with<'r>(&'r mut self, components: impl ComponentTuple<'r>) -> Entity {
        components.create_entity_with(self)
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

    pub fn add<Component: ComponentTrait>(&mut self, entity: Entity, new_component: Component) {
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

    pub fn remove<Component: ComponentTrait>(&mut self, entity: Entity) {
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

    pub fn replace<Component: ComponentTrait>(&mut self, entity: Entity, new_component: Component) {
        self.patch::<Component>(entity).with(move |component| *component = new_component);
    }

    pub fn patch<Component: ComponentTrait>(&mut self, entity: Entity) -> Patch<Component> {
        let component = self.component_pool.get_mut(&TypeId::of::<Component>()).and_then(|component_pool| {
            let component_storage = component_pool.as_any_mut().downcast_mut::<HashMap<Entity, Component>>().unwrap();
            component_storage.get_mut(&entity)
        });
        // TODO: notify observer here, instead of passing to Patch, why? that will allow multiple mut Patches
        Patch { observer: &mut self.observer, component }
    }

    pub fn get<Component: ComponentTrait>(&self, entity: Entity) -> Option<&Component> {
        self.component_pool.get(&TypeId::of::<Component>()).and_then(|component_pool| {
            let component_storage = component_pool.as_any().downcast_ref::<HashMap<Entity, Component>>().unwrap();
            component_storage.get(&entity)
        })
    }

    pub fn get_all<'r, Components: ComponentTuple<'r>>(&'r self, entity: Entity) -> Components::AsOption {
        Components::get_components(entity, self)
    }

    pub fn view_all<'r, Components: ComponentTuple<'r>>(&'r self) -> Vec<(Entity, Components::AsRef)> {
        Components::view_entities(self)
    }

    pub fn exists(&self, entity: Entity) -> bool {
        self.entities.contains_key(&entity)
    }

    // TODO: add_or_replace(component)
    // TODO: clear<component>()
}

pub struct Patch<'r, Component> {
    observer: &'r mut Observer,
    component: Option<&'r mut Component>,
}

impl<'r, Component> Patch<'r, Component> {
    pub fn with<F: FnOnce(&mut Component)>(&mut self, func: F) {
        if let Some(component) = &mut self.component {
            func(component);
            // TODO: Notify registry observer/event-manager
        }
    }
    // TODO: fn get_mut() ? should also notify the observer
}

pub trait ComponentTuple<'r> {
    type AsOption;
    type AsRef;
    fn create_entity_with(self, registry: &mut Registry) -> Entity;
    fn get_components(entity: Entity, registry: &'r Registry) -> Self::AsOption;
    fn view_entities(_registry: &'r Registry) -> Vec<(Entity, Self::AsRef)>;
}

// Reference: https://doc.rust-lang.org/1.5.0/src/core/tuple.rs.html#39-57
// FIXME(#19630) Remove this work-around
macro_rules! expr {
    ($e:expr) => { $e }
}

macro_rules! impl_component_tuple {
    ( $( $T:ident.$idx:tt ),+ ) => {
        impl<'r, $( $T ),+> ComponentTuple<'r> for ( $( $T, )+ )
            where $( $T: ComponentTrait ),+
        {
            type AsOption = ( $( Option<&'r $T>, )+ );
            type AsRef = ( $(&'r $T, )+ );

            fn create_entity_with(self, registry: &mut Registry) -> Entity {
                let entity = registry.create();
                $(
                    registry.add(entity, expr!(self.$idx));
                )+
                entity
            }

            fn get_components(entity: Entity, registry: &'r Registry) -> Self::AsOption {
                (
                    $(
                        registry.get::<$T>(entity),
                    )+
                )
            }

            fn view_entities(registry: &'r Registry) -> Vec<(Entity, Self::AsRef)> {
                let storages = (
                    $(
                        registry.component_pool.get(&TypeId::of::<$T>()).and_then(|component_pool| {
                            component_pool.as_any().downcast_ref::<HashMap<Entity, $T>>()
                        }),
                    )+
                );
                let storage_noexist = $( expr!(storages.$idx).is_none() )||+;
                if storage_noexist {
                    return Default::default();
                }
                let storages = ( $( expr!(storages.$idx).unwrap(), )+ );
                let storages = ( $( expr!(storages.$idx), )+ );
                let mut vec = Vec::new();
                for entity in storages.0.keys() {
                    let components = ( $( expr!(storages.$idx).get(&entity), )+ );
                    let exist = $( expr!(components.$idx).is_some() )&&+;
                    if exist {
                        let components = ( $( expr!(components.$idx).unwrap(), )+ );
                        vec.push((*entity, components));
                    }
                }
                vec
            }
        }
    }
}

macro_rules! impl_component_tuple_expand {
    ( $T:ident.$idx:tt ) => {
        impl_component_tuple!($T.$idx);
    };
    ( $T:ident.$idx:tt, $( $Ts:ident.$idxs:tt ),+ ) => {
        impl_component_tuple!($T.$idx, $( $Ts.$idxs ),+);
        impl_component_tuple_expand!($( $Ts.$idxs ),+);
    };
}

impl_component_tuple_expand!(L.11, K.10, J.9, I.8, H.7, G.6, F.5, E.4, D.3, C.2, B.1, A.0);
