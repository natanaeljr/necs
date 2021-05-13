use super::*;

struct Position { }

struct Velocity { }

#[test]
fn registry() {
    let mut registry = Registry::new();
    let entity = registry.create();
    registry.add(entity, Position{});
    registry.add(entity, Velocity{});
    println!("{:#?}", registry);
    registry.replace::<Position>(entity, Position{});
    registry.remove::<Velocity>(entity);
    registry.destroy(entity);
}

#[test]
fn handle() {
    let mut registry = Registry::new();
    let id = registry.create();
    let mut entity = Handle::new(&mut registry, id);
    assert_eq!(id, entity.id());
    entity.add(Position{});
    entity.add(Velocity{});
    entity.replace(Velocity{});
    entity.remove::<Position>();
}

#[test]
fn typeinfo() {
    assert_eq!("necs::tests::Position", std::any::type_name::<Position>());
    assert_eq!("necs::tests::Velocity", std::any::type_name::<Velocity>());
}
