#![allow(dead_code)]

use std::any::{Any, TypeId};
use std::collections::{HashSet, HashMap};
use std::collections::hash_map::Entry;

#[cfg(test)]
mod tests;

///////////////////////////////////////////////////////////////////////////////

type Entity = u64;
// TODO: Maybe make Entity = usize?
//  What are the advantages for the system/processor? Is it worth it?

const NULL_ENTITY: Entity = 0;

///////////////////////////////////////////////////////////////////////////////

type ComponentId = TypeId;

pub trait ComponentTrait: 'static + Sized {}

impl<T: 'static + Sized> ComponentTrait for T {}

///////////////////////////////////////////////////////////////////////////////

trait ComponentStorage {
    fn remove(&mut self, entity: &Entity);
    fn is_empty(&self) -> bool;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: ComponentTrait> ComponentStorage for HashMap<Entity, T> {
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

#[derive(Debug, Default)]
struct Observer;

///////////////////////////////////////////////////////////////////////////////
// TODO: ComponentStorage should be a Vector of Components.
//  For that, we need also a entity index redirection table (HashMap<Entity, Index> or another Vector?) to the vector of components.
//  Are Rust's HashMaps arrays internally? MUST KNOW

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

    fn create_with(&mut self, components: impl ComponentTuple) -> Entity {
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
        Patch { observer: &mut self.observer, component }
    }

    pub fn get<Component: ComponentTrait>(&self, entity: Entity) -> Option<&Component> {
        self.component_pool.get(&TypeId::of::<Component>()).and_then(|component_pool| {
            let component_storage = component_pool.as_any().downcast_ref::<HashMap<Entity, Component>>().unwrap();
            component_storage.get(&entity)
        })
    }

    pub fn get_all<'r, Components: ComponentSet<'r>>(&'r self, entity: Entity) -> Components::GetResult {
        Components::get_components(entity, self)
    }

    pub fn view<'r, Components: ComponentSet<'r>>(&'r self) -> Vec<(Entity, Components::ViewResult)> {
        Components::view_entities(self)
    }

    pub fn exists(&self, entity: Entity) -> bool {
        self.entities.contains_key(&entity)
    }
}

///////////////////////////////////////////////////////////////////////////////

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

///////////////////////////////////////////////////////////////////////////////

pub trait ComponentTuple {
    fn create_entity_with(self, registry: &mut Registry) -> Entity;
}

// Reference: https://doc.rust-lang.org/1.5.0/src/core/tuple.rs.html#39-57
// FIXME(#19630) Remove this work-around
macro_rules! e {
    ($e:expr) => { $e }
}

macro_rules! component_tuple {
    ( $( $T:ident.$idx:tt ),+ ) => {
        impl<$( $T ),+> ComponentTuple for ( $( $T, )+ )
            where $( $T: ComponentTrait ),+
        {
            fn create_entity_with(self, registry: &mut Registry) -> Entity {
                let entity = registry.create();
                $(
                    registry.add(entity, e!(self.$idx));
                )+
                entity
            }
        }
    }
}

component_tuple!(A.0);
component_tuple!(A.0, B.1);
component_tuple!(A.0, B.1, C.2);

pub trait ComponentSet<'r> {
    type GetResult: Default;
    type ViewResult;
    fn get_components(entity: Entity, registry: &'r Registry) -> Self::GetResult;
    fn view_entities(_registry: &'r Registry) -> Vec<(Entity, Self::ViewResult)> { Default::default() }
}

impl<'r, A, B> ComponentSet<'r> for (&A, &B) where A: ComponentTrait, B: ComponentTrait {
    type GetResult = (Option<&'r A>, Option<&'r B>);
    type ViewResult = (&'r A, &'r B);

    fn get_components(entity: Entity, registry: &'r Registry) -> Self::GetResult {
        (
            registry.get::<A>(entity),
            registry.get::<B>(entity),
        )
    }

    fn view_entities(registry: &'r Registry) -> Vec<(Entity, (&A, &B))> {
        let a_storage = registry.component_pool.get(&TypeId::of::<A>()).and_then(|component_pool| {
            component_pool.as_any().downcast_ref::<HashMap<Entity, A>>()
        });
        let b_storage = registry.component_pool.get(&TypeId::of::<B>()).and_then(|component_pool| {
            component_pool.as_any().downcast_ref::<HashMap<Entity, B>>()
        });
        if a_storage.is_none() || b_storage.is_none() {
            return Default::default();
        }
        let mut vec = Vec::new();
        let a_storage = a_storage.unwrap();
        let b_storage = b_storage.unwrap();
        for a in a_storage {
            if let Some(b) = b_storage.get(a.0) {
                vec.push((*a.0, (a.1, b)));
            }
        }
        vec
    }
}

macro_rules! tuple_ecs {
    ( $( $T:ident ),+ ) => {
        impl<'r, $( $T ),+> ComponentSet<'r> for ( $( &$T, )+ )
            where $( $T: ComponentTrait ),+
        {
            type GetResult = ( $( Option<&'r $T>, )+ );
            type ViewResult = ( $(&'r $T, )+ );

            fn get_components(entity: Entity, registry: &'r Registry) -> Self::GetResult {
                (
                    $(
                        registry.get::<$T>(entity),
                    )+
                )
            }
        }
    }
}

tuple_ecs!(A);
// tuple_ecs!(A, B);
tuple_ecs!(A, B, C);
tuple_ecs!(A, B, C, D);

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
    fn add<Component: ComponentTrait>(&mut self, new_component: Component) {
        self.registry.add(self.entity, new_component)
    }

    #[inline]
    fn remove<Component: ComponentTrait>(&mut self) {
        self.registry.remove::<Component>(self.entity)
    }

    #[inline]
    fn replace<Component: ComponentTrait>(&mut self, new_component: Component) {
        self.registry.replace(self.entity, new_component)
    }

    #[inline]
    fn patch<Component: ComponentTrait>(&mut self) -> Patch<Component> {
        self.registry.patch::<Component>(self.entity)
    }

    #[inline]
    fn get<Component: ComponentTrait>(&mut self) -> Option<&Component> {
        self.registry.get::<Component>(self.entity)
    }

    #[inline]
    pub fn get_all<'r, Components: ComponentSet<'r>>(&'r self) -> Components::GetResult {
        self.registry.get_all::<Components>(self.entity)
    }
}
