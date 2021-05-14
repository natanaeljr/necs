use super::*;

struct Position;

struct Velocity;

struct Color;

#[test]
fn registry() {
    let mut registry = Registry::new();
    let entity = registry.create();

    registry.add(entity, Position {});
    registry.add(entity, Velocity {});
    registry.add(entity, Color {});
    registry.replace(entity, Position {});
    registry.remove::<Color>(entity);

    assert!(!registry.exists(10));
    assert!(registry.get::<Position>(10).is_none());
    assert!(registry.get::<Velocity>(10).is_none());
    assert!(registry.get::<Color>(10).is_none());

    assert!(registry.exists(entity));
    assert!(registry.get::<Position>(entity).is_some());
    assert!(registry.get::<Velocity>(entity).is_some());
    assert!(registry.get::<Color>(entity).is_none());

    registry.destroy(entity);
    assert!(!registry.exists(entity));
    assert!(registry.get::<Position>(entity).is_none());
    assert!(registry.get::<Velocity>(entity).is_none());
    assert!(registry.get::<Color>(entity).is_none());
}

#[test]
fn handle() {
    let mut registry = Registry::new();
    let id = registry.create();
    let mut entity = Handle::new(&mut registry, id);
    assert_eq!(id, entity.id());
    entity.add(Position {});
    entity.add(Velocity {});
    entity.replace(Velocity {});
    entity.remove::<Position>();
}

#[test]
fn typeinfo() {
    assert_eq!("necs::tests::Position", std::any::type_name::<Position>());
    assert_eq!("necs::tests::Velocity", std::any::type_name::<Velocity>());
}
