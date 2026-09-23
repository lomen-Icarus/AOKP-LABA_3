//! Лабораторные работы № 1–3 по компьютерной графике (Rust / Bevy 0.19)
//! в одной программе. Лабы построены одна на другой: № 2 — доработка № 1,
//! № 3 использует модель, цвет `Vec3` и схему систем из № 1 и № 2.
//!
//! При запуске открывается лаба 3. Клавиши F1, F2, F3 выбирают лабу,
//! Tab переключает по кругу. Номер стартовой лабы можно передать
//! аргументом: `cargo run -- 1`.

use bevy::prelude::*;

// Модуль графического объекта из задания лабы 3.
mod graphic_object;
mod lab1;
mod lab2;
mod lab3;
#[cfg(test)]
mod tests;

/// Выбранная лабораторная работа. Каждая лаба создаёт свою сцену при входе
/// в это состояние, а сущности с `DespawnOnExit` удаляются при выходе.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lab {
    Lab1,
    Lab2,
    Lab3,
}

impl Lab {
    /// Разбор номера лабы из аргумента командной строки.
    pub fn from_arg(arg: &str) -> Option<Self> {
        match arg.trim() {
            "1" => Some(Self::Lab1),
            "2" => Some(Self::Lab2),
            "3" => Some(Self::Lab3),
            _ => None,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Lab1 => Self::Lab2,
            Self::Lab2 => Self::Lab3,
            Self::Lab3 => Self::Lab1,
        }
    }

    /// Заголовок окна: название лабы и её управление.
    pub fn title(self) -> &'static str {
        match self {
            Self::Lab1 => {
                "Лаба 1: цвет и вращение тора | Пробел, 1-5, A: авто | F1 F2 F3 / Tab: выбор лабы"
            }
            Self::Lab2 => {
                "Лаба 2: авто-смена цвета, glam и Vec | Пробел, 1-5, L: плавно | F1 F2 F3 / Tab"
            }
            Self::Lab3 => "Лаба 3: четыре объекта носиками к центру | R: вращение | F1 F2 F3 / Tab",
        }
    }
}

/// Подключает все три лабы и переключение между ними.
pub struct LabsPlugin {
    pub start: Lab,
}

impl Plugin for LabsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(self.start)
            .add_plugins((lab1::Lab01Plugin, lab2::Lab02Plugin, lab3::Lab03Plugin))
            .add_systems(Update, switch_lab_system)
            .add_systems(Update, update_window_title.run_if(state_changed::<Lab>));
    }
}

fn main() {
    let start = std::env::args()
        .nth(1)
        .and_then(|arg| Lab::from_arg(&arg))
        .unwrap_or(Lab::Lab3);
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: start.title().into(),
                resolution: (1000, 700).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(LabsPlugin { start })
        .run();
}

/// Выбор лабы: F1, F2, F3 — напрямую, Tab — следующая по кругу.
/// Новая лаба начинается со следующего кадра, как при отдельном запуске.
pub fn switch_lab_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    current: Res<State<Lab>>,
    mut next: ResMut<NextState<Lab>>,
) {
    let target = if keyboard.just_pressed(KeyCode::F1) {
        Some(Lab::Lab1)
    } else if keyboard.just_pressed(KeyCode::F2) {
        Some(Lab::Lab2)
    } else if keyboard.just_pressed(KeyCode::F3) {
        Some(Lab::Lab3)
    } else if keyboard.just_pressed(KeyCode::Tab) {
        Some(current.get().next())
    } else {
        None
    };
    if let Some(lab) = target.filter(|lab| lab != current.get()) {
        println!("[labs] переход: {:?} -> {:?}", current.get(), lab);
        next.set(lab);
    }
}

fn update_window_title(lab: Res<State<Lab>>, mut windows: Query<&mut Window>) {
    for mut window in &mut windows {
        window.title = lab.get().title().into();
    }
}
