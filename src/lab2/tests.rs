use super::*;

const WHITE: Vec3 = Vec3::new(1.0, 1.0, 1.0);
const BLUE: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const RED: Vec3 = Vec3::new(1.0, 0.0, 0.0);
const YELLOW: Vec3 = Vec3::new(1.0, 1.0, 0.0);
const PURPLE: Vec3 = Vec3::new(0.5, 0.0, 0.5);

fn test_app(step_ms: u64) -> App {
    super::super::tests::test_app(super::super::Lab::Lab2, step_ms)
}

fn index(app: &App) -> usize {
    app.world().resource::<ColorPalette>().current_index
}

fn elapsed(app: &App) -> f32 {
    app.world().resource::<Time>().elapsed_secs()
}

fn material_rgb(app: &mut App) -> Vec3 {
    let handle = app
        .world_mut()
        .query_filtered::<&MeshMaterial3d<StandardMaterial>, With<LabObject>>()
        .single(app.world())
        .expect("тор создан")
        .id();
    let c = app
        .world()
        .resource::<Assets<StandardMaterial>>()
        .get(handle)
        .expect("материал существует")
        .base_color
        .to_srgba();
    Vec3::new(c.red, c.green, c.blue)
}

fn press(app: &mut App, key: KeyCode) {
    super::super::tests::press(app, &[key]);
}

/// Продвигает время до ближайшей смены индекса и возвращает момент смены.
fn run_until_change(app: &mut App) -> f32 {
    let start = index(app);
    for _ in 0..1000 {
        app.update();
        if index(app) != start {
            return elapsed(app);
        }
    }
    panic!("цвет не сменился");
}

#[test]
fn palette_is_vec_of_vec3_in_required_order_and_wraps() {
    let mut palette = ColorPalette::new();
    assert_eq!(palette.colors, vec![WHITE, BLUE, RED, YELLOW, PURPLE]);
    for expected in [1, 2, 3, 4, 0, 1] {
        palette.next_color();
        assert_eq!(palette.current_index, expected);
    }
    palette.colors.push(Vec3::new(0.0, 1.0, 0.0));
    palette.select(4);
    palette.next_color();
    assert_eq!(
        palette.current_index, 5,
        "новый цвет входит в цикл без правок кода"
    );
    assert!(!palette.select(99));
}

#[test]
fn start_color_is_white_in_palette_and_material() {
    let mut app = test_app(100);
    assert_eq!(index(&app), 0);
    assert!(material_rgb(&mut app).abs_diff_eq(WHITE, 1e-6));
}

#[test]
fn color_changes_every_second_and_material_follows_in_same_frame() {
    let mut app = test_app(100);
    let mut previous = 0.0;
    for (step, expected) in [BLUE, RED, YELLOW, PURPLE, WHITE].into_iter().enumerate() {
        let at = run_until_change(&mut app);
        assert!(
            (at - previous - COLOR_INTERVAL_SECS).abs() < 0.11,
            "шаг {step}: {at}"
        );
        assert!(
            material_rgb(&mut app).abs_diff_eq(expected, 1e-6),
            "шаг {step}"
        );
        previous = at;
    }
}

#[test]
fn change_rate_does_not_depend_on_frame_rate() {
    for step_ms in [20, 50, 125] {
        let mut app = test_app(step_ms);
        let frames = 3_000 / step_ms;
        for _ in 0..frames {
            app.update();
        }
        assert_eq!(index(&app), 3, "шаг кадра {step_ms} мс");
    }
}

#[test]
fn long_frame_is_clamped_by_virtual_time() {
    // Res<Time> в Update — виртуальное время: за кадр не больше 250 мс.
    // Зависание на 2,5 с не «проматывает» палитру на два цвета вперёд.
    let mut app = test_app(2_500);
    let before = elapsed(&app);
    app.update();
    assert!((elapsed(&app) - before - 0.25).abs() < 1e-6);
    assert_eq!(index(&app), 0);
}

#[test]
fn space_and_digits_select_manually_and_restart_interval() {
    let mut app = test_app(250);
    for _ in 0..3 {
        app.update();
    }
    press(&mut app, KeyCode::Space);
    assert_eq!(index(&app), 1);
    let pressed_at = elapsed(&app);
    let next_auto = run_until_change(&mut app);
    assert!((next_auto - pressed_at - COLOR_INTERVAL_SECS).abs() < 0.26);
    press(&mut app, KeyCode::Digit5);
    assert_eq!(index(&app), 4);
    assert!(material_rgb(&mut app).abs_diff_eq(PURPLE, 1e-6));
    press(&mut app, KeyCode::Space);
    assert_eq!(index(&app), 0, "после фиолетового снова белый");
}

#[test]
fn smooth_mode_interpolates_and_settles_on_target() {
    let mut app = test_app(100);
    press(&mut app, KeyCode::KeyL);
    run_until_change(&mut app);
    // В кадре смены переход только начался (t ≈ 0); через 0,1 с цвет промежуточный.
    app.update();
    let mid = material_rgb(&mut app);
    assert!(
        mid.x < 1.0 && mid.y < 1.0 && mid.z > 0.99,
        "начало перехода белый → синий: {mid}"
    );
    for _ in 0..5 {
        app.update();
    }
    assert!(material_rgb(&mut app).abs_diff_eq(BLUE, 1e-6));
}

#[test]
fn rotation_angle_depends_on_time_not_frames() {
    let mut angles = Vec::new();
    for step_ms in [50, 100] {
        let mut app = test_app(step_ms);
        for _ in 0..(2_000 / step_ms) {
            app.update();
        }
        let transform = *app
            .world_mut()
            .query_filtered::<&Transform, With<LabObject>>()
            .single(app.world())
            .unwrap();
        angles.push(transform.rotation);
    }
    assert!(angles[0].abs_diff_eq(angles[1], 1e-4));
}
