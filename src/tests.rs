use super::*;

#[derive(Debug, Default, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug, Default, PartialEq)]
struct Velocity {
    dx: i32,
    dy: i32,
}

#[derive(Debug, Default, PartialEq)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

#[test]
fn registry() {
    let mut registry = Registry::new();
    let entity = registry.create();

    registry.add(entity, Position { x: 10, y: 20 });
    registry.add(entity, Velocity { dx: -50, dy: -100 });
    registry.add(entity, Color::default());
    registry.replace(entity, Position { x: 40, y: 80 });
    registry.remove::<Color>(entity);

    assert!(!registry.exists(10));
    assert!(registry.get::<Position>(10).is_none());
    assert!(registry.get::<Velocity>(10).is_none());
    assert!(registry.get::<Color>(10).is_none());

    assert!(registry.exists(entity));
    assert_eq!(registry.get::<Position>(entity), Some(&Position { x: 40, y: 80 }));
    assert_eq!(registry.get::<Velocity>(entity), Some(&Velocity { dx: -50, dy: -100 }));
    assert_eq!(registry.get::<Color>(entity), None);

    registry.patch::<Velocity>(entity).with(|vel| vel.dy -= 20);
    assert_ne!(registry.get::<Velocity>(entity), Some(&Velocity { dx: -50, dy: -100 }));
    assert_eq!(registry.get::<Velocity>(entity), Some(&Velocity { dx: -50, dy: -120 }));

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
    entity.add(Position::default());
    entity.add(Velocity::default());
    entity.replace(Velocity::default());
    entity.remove::<Position>();
    entity.patch::<Velocity>().with(|vel| vel.dy -= 20);
}

#[test]
fn typeinfo() {
    assert_eq!("necs::tests::Position", std::any::type_name::<Position>());
    assert_eq!("necs::tests::Velocity", std::any::type_name::<Velocity>());
}

#[test]
fn get_tuple() {
    let mut registry = Registry::new();
    let entity = registry.create();
    registry.add(entity, Position::default());
    registry.add(entity, Velocity::default());
    registry.add(entity, Color::default());
    registry.replace(entity, Position::default());
    registry.remove::<Color>(entity);

    let (position, velocity, color) = registry.get_all::<(&Position, &Velocity, &Color)>(entity);
    assert!(position.is_some());
    assert!(velocity.is_some());
    assert!(color.is_none());

    let (position, velocity) = registry.get_all::<(&Position, &Velocity)>(entity);
    assert!(position.is_some());
    assert!(velocity.is_some());

    let (position, velocity) = <(&Position, &Velocity)>::get_components(entity, &registry);
    assert!(position.is_some());
    assert!(velocity.is_some());

    let (color, ) = <(&Color, )>::get_components(entity, &registry);
    assert!(color.is_none());
}

#[test]
fn view() {
    let mut registry = Registry::new();
    let entity = registry.create();
    registry.add(entity, Position { x: 10, y: 20 });
    registry.add(entity, Velocity { dx: -50, dy: -100 });
    registry.add(entity, Color::default());
    let entity = registry.create();
    registry.add(entity, Position { x: 20, y: 30 });
    registry.add(entity, Velocity { dx: -60, dy: -200 });
    let entity = registry.create();
    registry.add(entity, Position { x: 20, y: 30 });
    registry.add(entity, Color::default());


    let all = <(&Position, &Velocity)>::view_entities(&registry);
    println!("{:?}", all);

    for (entt, (position, velocity)) in registry.view_entities_with::<(&Position, &Velocity)>() {
        println!("{:?}", entt);
    }

    registry.view_entities_with::<(&Position, &Velocity)>().iter().for_each(|(entt, (pos, vel))| {
        println!("{:?}", entt);
    });

    assert!(false)
}
