//! Кадры окна приложения для отчёта: пять цветов палитры и кадр плавного перехода.
//! Клавиши подаются через ButtonInput перед keyboard_system, как настоящий ввод.
//! Запуск: cargo run --example capture (на Linux без экрана — под xvfb-run).

use bevy::prelude::*;
use bevy::render::view::window::screenshot::{Screenshot, save_to_disk};
use bevy_lab02_color_cycle::{Lab02Plugin, keyboard_system};

/// (кадр нажатия, клавиша, имя файла). Снимок делается через 4 кадра после нажатия.
const SHOTS: [(u32, KeyCode, &str); 5] = [
    (30, KeyCode::Digit1, "lab02_1_white.png"),
    (45, KeyCode::Digit2, "lab02_2_blue.png"),
    (60, KeyCode::Digit3, "lab02_3_red.png"),
    (75, KeyCode::Digit4, "lab02_4_yellow.png"),
    (90, KeyCode::Digit5, "lab02_5_purple.png"),
];
const SMOOTH_ON_FRAME: u32 = 105;
const SMOOTH_SWITCH_FRAME: u32 = 110;
const SMOOTH_SHOT_FRAME: u32 = 119;
const EXIT_FRAME: u32 = 140;
const SHOT_DELAY: u32 = 4;

#[derive(Resource, Default)]
struct FrameCounter(u32);

fn main() {
    std::fs::create_dir_all("screenshots").expect("каталог screenshots");
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Lab 02 | capture".into(),
                resolution: (1000, 700).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.22, 0.88, 0.11)))
        .add_plugins(Lab02Plugin)
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
    let frame = counter.0;
    keyboard.release_all();
    for (press_frame, key, file) in SHOTS {
        if frame == press_frame {
            keyboard.press(key);
        }
        if frame == press_frame + SHOT_DELAY {
            shot(&mut commands, file);
        }
    }
    match frame {
        SMOOTH_ON_FRAME => keyboard.press(KeyCode::KeyL),
        SMOOTH_SWITCH_FRAME => keyboard.press(KeyCode::Space),
        SMOOTH_SHOT_FRAME => shot(&mut commands, "lab02_6_transition.png"),
        EXIT_FRAME => {
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}

fn shot(commands: &mut Commands, file: &str) {
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(format!("screenshots/{file}")));
}
