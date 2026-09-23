//! Получение кадров реального окна приложения: сначала сцена из задания,
//! затем та же сцена после нажатия R (обращение вокруг центра).
//! Запуск: cargo run --example capture (при необходимости под xvfb-run).

use bevy::prelude::*;
use bevy::render::view::window::screenshot::{Screenshot, save_to_disk};
use bevy_lab03_scene_objects::{Lab03Plugin, keyboard_system};

const SCENE_FRAME: u32 = 40;
const ORBIT_ON_FRAME: u32 = 60;
const ORBIT_FRAME: u32 = 120;
const EXIT_FRAME: u32 = 180;

#[derive(Resource, Default)]
struct FrameCounter(u32);

fn main() {
    std::fs::create_dir_all("screenshots").expect("каталог screenshots");
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Lab 03 | capture".into(),
                resolution: (1000, 700).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(Lab03Plugin)
        .init_resource::<FrameCounter>()
        .add_systems(Update, capture_system.before(keyboard_system))
        .run();
}

fn capture_system(
    mut commands: Commands,
    mut counter: ResMut<FrameCounter>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut exit: MessageWriter<AppExit>,
) {
    counter.0 += 1;
    keyboard.release_all();
    match counter.0 {
        SCENE_FRAME => {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk("screenshots/lab03_scene.png"));
        }
        // Нажатие R проходит через keyboard_system, как настоящий ввод.
        ORBIT_ON_FRAME => keyboard.press(KeyCode::KeyR),
        ORBIT_FRAME => {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk("screenshots/lab03_orbit.png"));
        }
        EXIT_FRAME => {
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}
