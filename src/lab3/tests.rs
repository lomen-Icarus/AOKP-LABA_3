use super::*;

const EPS: f32 = 1e-4;

fn test_app() -> App {
    super::super::tests::test_app(super::super::Lab::Lab3, 100)
}

/// Направление носика: локальная ось +X после поворота объекта.
fn nose(transform: &Transform) -> Vec3 {
    transform.rotation * Vec3::X
}

fn transforms(app: &mut App) -> Vec<(Vec3, Transform)> {
    app.world_mut()
        .query::<(&GraphicObject, &Transform)>()
        .iter(app.world())
        .map(|(object, transform)| (object.color, *transform))
        .collect()
}

#[test]
fn four_noses_point_to_scene_center() {
    let scene = scene_objects();
    assert_eq!(scene.len(), 4);
    for object in scene {
        let to_center = -object.position.normalize();
        assert!(
            nose(&object.to_transform()).abs_diff_eq(to_center, EPS),
            "{object:?}"
        );
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
fn scene_has_four_objects_with_their_transforms() {
    let mut app = test_app();
    let spawned = transforms(&mut app);
    assert_eq!(spawned.len(), 4);
    for object in scene_objects() {
        let (_, transform) = spawned
            .iter()
            .find(|(color, _)| *color == object.color)
            .expect("объект такого цвета создан");
        assert!(transform.translation.abs_diff_eq(object.position, EPS));
        assert!(
            transform
                .rotation
                .abs_diff_eq(object.to_transform().rotation, EPS)
        );
    }
    // У каждого объекта тор и носик.
    let meshes = app.world_mut().query::<&Mesh3d>().iter(app.world()).count();
    assert_eq!(meshes, 8);
}

#[test]
fn orbit_is_off_until_r_is_pressed() {
    let mut app = test_app();
    let before = transforms(&mut app);
    app.update();
    app.update();
    for (color, transform) in transforms(&mut app) {
        let (_, start) = before.iter().find(|(c, _)| *c == color).unwrap();
        assert!(transform.translation.abs_diff_eq(start.translation, EPS));
    }
}

#[test]
fn orbit_keeps_noses_toward_center() {
    let mut app = test_app();
    super::super::tests::press(&mut app, &[KeyCode::KeyR]);
    for _ in 0..30 {
        app.update();
    }
    for object in scene_objects() {
        let (_, transform) = transforms(&mut app)
            .into_iter()
            .find(|(color, _)| *color == object.color)
            .unwrap();
        assert!((transform.translation.length() - 4.0).abs() < 1e-3);
        assert!(
            !transform.translation.abs_diff_eq(object.position, 1e-2),
            "объект не сдвинулся"
        );
        let to_center = -transform.translation.normalize();
        assert!(nose(&transform).dot(to_center) > 0.999);
    }
}
