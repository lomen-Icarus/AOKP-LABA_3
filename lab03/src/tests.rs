use std::time::Duration;

use bevy::time::TimeUpdateStrategy;

use super::*;

const EPS: f32 = 1e-4;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<ButtonInput<KeyCode>>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )))
        .add_plugins(Lab03Plugin);
    app
}

fn objects(app: &mut App) -> Vec<(GraphicObject, Transform)> {
    app.world_mut()
        .query::<(&GraphicObject, &Transform)>()
        .iter(app.world())
        .map(|(object, transform)| (object.clone(), *transform))
        .collect()
}

#[test]
fn four_noses_point_to_scene_center() {
    let scene = scene_objects();
    assert_eq!(scene.len(), 4);
    let expected = [-Vec3::X, Vec3::X, Vec3::Z, -Vec3::Z];
    for (object, direction) in scene.iter().zip(expected) {
        assert!(
            object.nose_direction().abs_diff_eq(direction, EPS),
            "{object:?}"
        );
        assert!(object.faces(Vec3::ZERO), "{object:?}");
    }
}

#[test]
fn transform_is_translation_and_clockwise_rotation() {
    let object = GraphicObject::new(Vec3::new(0.0, 0.0, -4.0), 90.0, Vec3::ONE);
    let transform = object.to_transform();
    assert!(transform.translation.abs_diff_eq(object.position, EPS));
    assert!(
        transform
            .rotation
            .abs_diff_eq(Quat::from_rotation_y(-90f32.to_radians()), EPS)
    );
    assert!(transform.scale.abs_diff_eq(Vec3::ONE, EPS));
    // Вершина модели (1, 0, 0) после матрицы модели оказывается ближе к центру.
    let world_point = transform.transform_point(Vec3::X);
    assert!(world_point.abs_diff_eq(Vec3::new(0.0, 0.0, -3.0), EPS));
}

#[test]
fn color_channels_are_preserved() {
    let object = GraphicObject::new(Vec3::ZERO, 0.0, Vec3::new(1.0, 0.0, 0.0));
    let srgba = object.to_color().to_srgba();
    assert_eq!(
        (srgba.red, srgba.green, srgba.blue, srgba.alpha),
        (1.0, 0.0, 0.0, 1.0)
    );
}

#[test]
fn startup_spawns_four_objects_sharing_one_model() {
    let mut app = test_app();
    app.update();
    let spawned = objects(&mut app);
    assert_eq!(spawned.len(), 4);
    for (object, transform) in &spawned {
        assert!(
            transform
                .translation
                .abs_diff_eq(object.to_transform().translation, EPS)
        );
        assert!(
            transform
                .rotation
                .abs_diff_eq(object.to_transform().rotation, EPS)
        );
    }
    let colors: Vec<Vec3> = spawned.iter().map(|(object, _)| object.color).collect();
    assert!(colors.contains(&Vec3::new(1.0, 0.0, 0.0)));
    assert!(colors.contains(&Vec3::new(0.0, 0.0, 1.0)));
    assert!(colors.contains(&Vec3::new(0.0, 1.0, 0.0)));
    assert!(colors.contains(&Vec3::new(1.0, 1.0, 1.0)));
    // Два меша (тор и носик) на четыре объекта: модель хранится один раз.
    assert_eq!(app.world().resource::<Assets<Mesh>>().len(), 2);
    assert_eq!(app.world().resource::<Assets<StandardMaterial>>().len(), 4);
    let mesh_entities = app.world_mut().query::<&Mesh3d>().iter(app.world()).count();
    assert_eq!(mesh_entities, 8);
}

/// Объект с тем же цветом: цвета уникальны, порядок запроса Bevy не гарантирует.
fn same_color(list: &[(GraphicObject, Transform)], color: Vec3) -> &(GraphicObject, Transform) {
    list.iter()
        .find(|(object, _)| object.color == color)
        .expect("объект с таким цветом есть")
}

fn press_r(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyR);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
}

fn material_change_tick(app: &App) -> u32 {
    app.world()
        .get_resource_change_ticks::<Assets<StandardMaterial>>()
        .expect("ресурс материалов есть")
        .changed
        .get()
}

#[test]
fn orbit_is_off_until_r_is_pressed() {
    let mut app = test_app();
    app.update();
    let before = objects(&mut app);
    app.update();
    app.update();
    let after = objects(&mut app);
    for (object, _) in &before {
        assert_eq!(object, &same_color(&after, object.color).0);
    }
}

#[test]
fn orbit_keeps_noses_toward_center() {
    let mut app = test_app();
    app.update();
    press_r(&mut app);
    for _ in 0..30 {
        app.update();
    }
    let moved = objects(&mut app);
    for start in scene_objects() {
        let (object, transform) = same_color(&moved, start.color);
        assert!(object.faces(Vec3::ZERO), "{object:?}");
        assert!((object.position.length() - start.position.length()).abs() < EPS);
        assert!(
            !object.position.abs_diff_eq(start.position, 1e-2),
            "объект не сдвинулся"
        );
        assert!(transform.translation.abs_diff_eq(object.position, EPS));
    }
}

#[test]
fn long_orbit_does_not_drift() {
    // 3000 кадров по 0,1 с = 300 с, почти девять полных оборотов.
    let mut app = test_app();
    app.update();
    press_r(&mut app);
    for _ in 0..3000 {
        app.update();
    }
    for (object, _) in objects(&mut app) {
        assert!((object.position.length() - 4.0).abs() < 1e-4, "{object:?}");
        assert!(object.position.y.abs() < 1e-4);
        assert!(object.faces(Vec3::ZERO), "{object:?}");
    }
}

#[test]
fn orbit_does_not_rewrite_materials() {
    let mut app = test_app();
    app.update();
    press_r(&mut app);
    app.update();
    let tick = material_change_tick(&app);
    for _ in 0..10 {
        app.update();
    }
    assert_eq!(
        material_change_tick(&app),
        tick,
        "цвет не менялся — материалы не трогаем"
    );
}

#[test]
fn changing_object_fields_recalculates_transform_and_color() {
    let mut app = test_app();
    app.update();
    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<GraphicObject>>()
        .iter(app.world())
        .next()
        .expect("объект создан");
    {
        let mut object = app.world_mut().get_mut::<GraphicObject>(entity).unwrap();
        object.position = Vec3::new(0.0, 2.0, 0.0);
        object.angle_deg = 45.0;
        object.color = Vec3::new(0.0, 1.0, 1.0);
    }
    app.update();
    let object = app.world().get::<GraphicObject>(entity).unwrap().clone();
    let transform = *app.world().get::<Transform>(entity).unwrap();
    assert!(
        transform
            .translation
            .abs_diff_eq(Vec3::new(0.0, 2.0, 0.0), EPS)
    );
    assert!(
        transform
            .rotation
            .abs_diff_eq(object.to_transform().rotation, EPS)
    );
    let handle = app
        .world()
        .get::<MeshMaterial3d<StandardMaterial>>(entity)
        .unwrap()
        .id();
    let material = app
        .world()
        .resource::<Assets<StandardMaterial>>()
        .get(handle)
        .unwrap();
    let srgba = material.base_color.to_srgba();
    assert_eq!((srgba.red, srgba.green, srgba.blue), (0.0, 1.0, 1.0));
}

#[test]
fn quarter_turn_moves_red_into_green_slot() {
    let red = GraphicObject::new(Vec3::new(4.0, 0.0, 0.0), 180.0, Vec3::X).orbited(90.0);
    assert!(red.position.abs_diff_eq(Vec3::new(0.0, 0.0, -4.0), EPS));
    assert!((red.angle_deg - 90.0).abs() < EPS);
    assert!(red.faces(Vec3::ZERO));
}
