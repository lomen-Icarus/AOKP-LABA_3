use super::super::tests::press;
use super::*;

fn test_app(step_ms: u64) -> App {
    super::super::tests::test_app(super::super::Lab::Lab1, step_ms)
}

fn index(app: &App) -> usize {
    app.world().resource::<ColorPalette>().current_index
}

fn material_color(app: &mut App) -> Color {
    let handle = app
        .world_mut()
        .query_filtered::<&MeshMaterial3d<StandardMaterial>, With<LabObject>>()
        .single(app.world())
        .expect("тор создан")
        .id();
    app.world()
        .resource::<Assets<StandardMaterial>>()
        .get(handle)
        .expect("материал существует")
        .base_color
}

#[test]
fn start_color_matches_material_and_space_cycles_all_colors() {
    let mut app = test_app(16);
    let palette = app.world().resource::<ColorPalette>().clone_colors();
    assert_eq!(material_color(&mut app), palette[0]);
    for expected in [1, 2, 3, 4, 0] {
        press(&mut app, &[KeyCode::Space]);
        assert_eq!(index(&app), expected);
        assert_eq!(material_color(&mut app), palette[expected]);
    }
}

#[test]
fn digits_select_directly_and_win_over_space() {
    let mut app = test_app(16);
    press(&mut app, &[KeyCode::Digit5]);
    assert_eq!(index(&app), 4);
    press(&mut app, &[KeyCode::Space]);
    assert_eq!(index(&app), 0, "после фиолетового — чёрный");
    press(&mut app, &[KeyCode::Space, KeyCode::Digit3]);
    assert_eq!(index(&app), 2, "цифра приоритетнее пробела");
}

#[test]
fn key_a_starts_and_stops_one_second_timer() {
    let mut app = test_app(250);
    for _ in 0..8 {
        app.update();
    }
    assert_eq!(index(&app), 0, "автоматика выключена при старте");
    press(&mut app, &[KeyCode::KeyA]);
    for _ in 0..8 {
        app.update();
    }
    assert_eq!(index(&app), 2, "за 2 с — две смены");
    press(&mut app, &[KeyCode::KeyA]);
    for _ in 0..8 {
        app.update();
    }
    assert_eq!(index(&app), 2, "после выключения цвет не меняется");
}

#[test]
fn manual_action_on_timer_boundary_gives_single_step_and_full_second() {
    let mut app = test_app(250);
    press(&mut app, &[KeyCode::KeyA]);
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(index(&app), 0);
    // В этом кадре таймер завершил бы секунду, но ручной выбор важнее.
    press(&mut app, &[KeyCode::Digit2]);
    assert_eq!(index(&app), 1, "только ручной шаг, без автоматического");
    for _ in 0..3 {
        app.update();
    }
    assert_eq!(index(&app), 1, "новый цвет держится полную секунду");
    app.update();
    assert_eq!(index(&app), 2);
}

#[test]
fn rotation_depends_on_time_not_frame_count() {
    let mut rotations = Vec::new();
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
        rotations.push(transform.rotation);
    }
    assert!(rotations[0].abs_diff_eq(rotations[1], 1e-4));
}

impl ColorPalette {
    fn clone_colors(&self) -> Vec<Color> {
        self.colors.clone()
    }
}
