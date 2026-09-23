//! Кадры окна программы для отчётов: проходит лабы 1, 2 и 3 подряд.
//! Клавиши подаются в ButtonInput сразу после обработки ввода Bevy,
//! поэтому программа получает их так же, как нажатия на клавиатуре.
//! Запуск: cargo run --example capture (на Linux без экрана — под xvfb-run).

use bevy::input::InputSystems;
use bevy::prelude::*;
use bevy::render::view::window::screenshot::{Screenshot, save_to_disk};

// Подключаем главный файл программы как модуль: те же лабы, те же системы.
// Его собственная функция main здесь не вызывается.
#[allow(dead_code)]
#[path = "../src/main.rs"]
mod app;

use app::{Lab, LabsPlugin};

enum Step {
    Press(KeyCode),
    Shot(&'static str),
    Exit,
}

/// Сценарий: номер кадра и действие.
const SCRIPT: &[(u32, Step)] = &[
    // Лаба 1: прямой выбор цвета цифрами.
    (20, Step::Press(KeyCode::Digit3)),
    (24, Step::Shot("lab01_blue.png")),
    (35, Step::Press(KeyCode::Digit4)),
    (39, Step::Shot("lab01_red.png")),
    (50, Step::Press(KeyCode::Digit5)),
    (54, Step::Shot("lab01_purple.png")),
    // Лаба 2: все цвета палитры и плавный переход.
    (65, Step::Press(KeyCode::F2)),
    (75, Step::Press(KeyCode::Digit1)),
    (79, Step::Shot("lab02_1_white.png")),
    (90, Step::Press(KeyCode::Digit2)),
    (94, Step::Shot("lab02_2_blue.png")),
    (105, Step::Press(KeyCode::Digit3)),
    (109, Step::Shot("lab02_3_red.png")),
    (120, Step::Press(KeyCode::Digit4)),
    (124, Step::Shot("lab02_4_yellow.png")),
    (135, Step::Press(KeyCode::Digit5)),
    (139, Step::Shot("lab02_5_purple.png")),
    (150, Step::Press(KeyCode::KeyL)),
    (155, Step::Press(KeyCode::Space)),
    (164, Step::Shot("lab02_6_transition.png")),
    // Лаба 3: сцена из задания и обращение вокруг центра.
    (175, Step::Press(KeyCode::F3)),
    (195, Step::Shot("lab03_scene.png")),
    (210, Step::Press(KeyCode::KeyR)),
    (270, Step::Shot("lab03_orbit.png")),
    (300, Step::Exit),
];

#[derive(Resource, Default)]
struct FrameCounter(u32);

fn main() {
    std::fs::create_dir_all("screenshots").expect("каталог screenshots");
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: Lab::Lab1.title().into(),
                resolution: (1000, 700).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(LabsPlugin { start: Lab::Lab1 })
        .init_resource::<FrameCounter>()
        .add_systems(PreUpdate, capture_system.after(InputSystems))
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
    for (frame, step) in SCRIPT {
        if *frame != counter.0 {
            continue;
        }
        match step {
            Step::Press(key) => keyboard.press(*key),
            Step::Shot(file) => {
                commands
                    .spawn(Screenshot::primary_window())
                    .observe(save_to_disk(format!("screenshots/{file}")));
            }
            Step::Exit => {
                exit.write(AppExit::Success);
            }
        }
    }
}
