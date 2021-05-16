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

#[test]
fn gets() {
    let mut registry = Registry::new();
    let entity = registry.create();
    registry.add(entity, Position {});
    registry.add(entity, Velocity {});
    registry.add(entity, Color {});
    registry.replace(entity, Position {});
    registry.remove::<Color>(entity);

    let (position, velocity, color) = registry.get_all::<(Position, Velocity, Color)>(entity);
    assert!(position.is_some());
    assert!(velocity.is_some());
    assert!(color.is_none());

    let (position, velocity) = registry.get_all::<(Position, Velocity)>(entity);
    assert!(position.is_some());
    assert!(velocity.is_some());

    let (position, velocity) = <(Position, Velocity)>::get_components(entity, &registry);
    assert!(position.is_some());
    assert!(velocity.is_some());

    let (color, ) = <(Color, )>::get_components(entity, &registry);
    assert!(color.is_none());
}
