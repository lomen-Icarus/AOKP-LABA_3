//! Общий помощник тестов и тесты переключения лаб.

use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use super::*;

/// Приложение без окна и рендера с выбранной стартовой лабой и
/// фиксированной длительностью кадра. Первый кадр уже выполнен.
pub fn test_app(start: Lab, step_ms: u64) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<ButtonInput<KeyCode>>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            step_ms,
        )))
        .add_plugins(LabsPlugin { start });
    app.update();
    app
}

/// Нажатие клавиш на один кадр.
pub fn press(app: &mut App, keys: &[KeyCode]) {
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    for key in keys {
        input.press(*key);
    }
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
}

fn current(app: &App) -> Lab {
    *app.world().resource::<State<Lab>>().get()
}

fn count<C: Component>(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<(), With<C>>()
        .iter(app.world())
        .count()
}

fn clear_color(app: &App) -> Color {
    app.world().resource::<ClearColor>().0
}

#[test]
fn program_starts_with_lab3_scene_only() {
    let mut app = test_app(Lab::Lab3, 16);
    assert_eq!(current(&app), Lab::Lab3);
    assert_eq!(count::<graphic_object::GraphicObject>(&mut app), 4);
    assert_eq!(count::<lab1::LabObject>(&mut app), 0);
    assert_eq!(count::<lab2::LabObject>(&mut app), 0);
    assert_eq!(count::<Camera3d>(&mut app), 1);
    assert_eq!(clear_color(&app), Color::BLACK);
}

#[test]
fn function_keys_switch_scenes_and_clean_up_previous_lab() {
    let mut app = test_app(Lab::Lab3, 16);
    press(&mut app, &[KeyCode::F1]);
    app.update();
    assert_eq!(current(&app), Lab::Lab1);
    assert_eq!(count::<graphic_object::GraphicObject>(&mut app), 0);
    assert_eq!(count::<lab1::LabObject>(&mut app), 1);
    assert_eq!(
        count::<Camera3d>(&mut app),
        1,
        "камера прежней лабы удалена"
    );
    assert_eq!(count::<Mesh3d>(&mut app), 1, "торы и носики лабы 3 удалены");
    assert_eq!(clear_color(&app), Color::srgb(0.22, 0.88, 0.11));

    press(&mut app, &[KeyCode::F2]);
    app.update();
    assert_eq!(current(&app), Lab::Lab2);
    assert_eq!(count::<lab1::LabObject>(&mut app), 0);
    assert_eq!(count::<lab2::LabObject>(&mut app), 1);

    press(&mut app, &[KeyCode::F3]);
    app.update();
    assert_eq!(current(&app), Lab::Lab3);
    assert_eq!(count::<graphic_object::GraphicObject>(&mut app), 4);
    assert_eq!(count::<Camera3d>(&mut app), 1);
    assert_eq!(clear_color(&app), Color::BLACK);
}

#[test]
fn tab_cycles_labs_in_order() {
    let mut app = test_app(Lab::Lab1, 16);
    for expected in [Lab::Lab2, Lab::Lab3, Lab::Lab1] {
        press(&mut app, &[KeyCode::Tab]);
        app.update();
        assert_eq!(current(&app), expected);
    }
}

#[test]
fn reentering_lab_starts_it_fresh() {
    let mut app = test_app(Lab::Lab2, 16);
    press(&mut app, &[KeyCode::Digit4]);
    assert_eq!(
        app.world().resource::<lab2::ColorPalette>().current_index,
        3
    );
    press(&mut app, &[KeyCode::F1]);
    app.update();
    press(&mut app, &[KeyCode::F2]);
    app.update();
    assert_eq!(
        app.world().resource::<lab2::ColorPalette>().current_index,
        0
    );
    assert_eq!(count::<lab2::LabObject>(&mut app), 1);
}

#[test]
fn start_lab_is_parsed_from_argument() {
    assert_eq!(Lab::from_arg("1"), Some(Lab::Lab1));
    assert_eq!(Lab::from_arg(" 2 "), Some(Lab::Lab2));
    assert_eq!(Lab::from_arg("3"), Some(Lab::Lab3));
    assert_eq!(Lab::from_arg("x"), None);
}
