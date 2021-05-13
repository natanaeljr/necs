use std::collections::HashSet;
use std::any::TypeId;

#[cfg(test)]
mod tests;

///////////////////////////////////////////////////////////////////////////////

type Entity = u64;

const NULL_ENTITY: Entity = 0;

///////////////////////////////////////////////////////////////////////////////
// Component has to be ?Sized and 'static

struct Registry {
    next: Entity,
    entities: HashMap<Entity, HashSet<TypeId>>,
    // components: HashMap<TypeId, HashMap<Entity, 
}

impl Registry {
    pub fn new() -> Self {
        Self{ next: 1, entities: Vec::new() }
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

    pub fn add<Component>(&mut self, entity: Entity, component: Component) -> bool {
        if let Some(value) = self.entities.get(&entity) {
            value.insert(TypeId::of::<Component>());
        }
        false
    }

    pub fn remove<Component>(&mut self, entity: Entity) {
        
    }

    pub fn replace<Component>(&mut self, entity: Entity, component: Component) {
        
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

    fn add<Component>(&mut self, component: Component) {
    }

    fn remove<Component>(&mut self) {
    }

    fn replace<Component>(&mut self, component: Component) {
    }
}
