//! Лабораторная работа №1: инициализация Bevy и ECS.
//! Тор меняет цвет по пробелу и цифрам 1–5, клавиша A включает смену
//! раз в секунду, тор вращается вокруг Y с учётом времени кадра.
//! В единой программе лаба активна, когда выбрана клавишей F1.

use bevy::prelude::*;

use super::Lab;

#[cfg(test)]
mod tests;

const ROTATION_SPEED: f32 = 0.7; // Радианы в секунду.
const COLOR_KEYS: [KeyCode; 5] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
];

/// Маркер объекта лабораторной: отличает тор от камеры и света в запросах.
#[derive(Component)]
pub struct LabObject;

/// Общая палитра хранится один раз в мире ECS.
/// Индекс используется и ручным, и автоматическим выбором цвета.
#[derive(Resource)]
pub struct ColorPalette {
    colors: Vec<Color>,
    current_index: usize,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            colors: vec![
                Color::srgb(0.0, 0.0, 0.0), // Чёрный.
                Color::srgb(1.0, 1.0, 1.0), // Белый.
                Color::srgb(0.0, 0.0, 1.0), // Синий.
                Color::srgb(1.0, 0.0, 0.0), // Красный.
                Color::srgb(0.5, 0.0, 0.5), // Фиолетовый.
            ],
            current_index: 0,
        }
    }
}

impl ColorPalette {
    fn next(&mut self) {
        self.current_index = (self.current_index + 1) % self.colors.len();
    }

    fn current(&self) -> Color {
        self.colors[self.current_index]
    }
}

/// Таймер считает игровой интервал, skip_tick защищает ручной ввод.
/// Эти данные общие для систем клавиатуры и автоматической смены.
#[derive(Resource)]
struct AutoColorTimer {
    timer: Timer,
    skip_tick: bool,
}

/// Режим автоматической смены. Подсостояние лабы 1: существует только
/// пока выбрана лаба 1 и при каждом входе в неё начинается с `Off`.
#[derive(SubStates, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[source(Lab = Lab::Lab1)]
pub enum AutoColorState {
    #[default]
    Off,
    On,
}

pub struct Lab01Plugin;

/// Плагин связывает ресурсы, состояния и системы.
/// chain задаёт порядок: клавиатура, таймер, материал.
impl Plugin for Lab01Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColorPalette>()
            .insert_resource(AutoColorTimer {
                timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                skip_tick: false,
            })
            .add_sub_state::<AutoColorState>()
            // При каждом входе в лабу 1 сцена и данные создаются заново.
            .add_systems(OnEnter(Lab::Lab1), (reset_lab1, setup_torus).chain())
            .add_systems(
                Update,
                (
                    keyboard_input_system,
                    auto_color_system.run_if(in_state(AutoColorState::On)),
                    apply_color_system,
                )
                    .chain()
                    .run_if(in_state(Lab::Lab1)),
            )
            .add_systems(Update, rotation_system.run_if(in_state(Lab::Lab1)));
    }
}

/// Начальное состояние лабы 1: чёрный цвет, таймер с нуля, зелёный фон.
fn reset_lab1(mut commands: Commands) {
    commands.insert_resource(ColorPalette::default());
    commands.insert_resource(AutoColorTimer {
        timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        skip_tick: false,
    });
    commands.insert_resource(ClearColor(Color::srgb(0.22, 0.88, 0.11)));
    commands.insert_resource(GlobalAmbientLight::default());
}

/// Эта система выполняется при входе в лабу: создаёт тор, камеру и свет.
/// Геометрия и материал помещаются в Assets, сущность хранит handles.
fn setup_torus(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<ColorPalette>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Torus::new(0.4, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: palette.current(),
            perceptual_roughness: 0.35,
            ..default()
        })),
        // Наклон делает вращение вокруг мировой Y заметным даже без текстуры.
        Transform::from_rotation(Quat::from_rotation_x(0.65)),
        LabObject,
        DespawnOnExit(Lab::Lab1),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.8, 2.0, 3.5).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(Lab::Lab1),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_xyz(3.0, 5.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(Lab::Lab1),
    ));
}

/// just_pressed обрабатывает новое нажатие, а не удержание.
/// Изменяем индекс; материал обновит отдельная следующая система.
fn keyboard_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut palette: ResMut<ColorPalette>,
    mut timer: ResMut<AutoColorTimer>,
    state: Res<State<AutoColorState>>,
    mut next_state: ResMut<NextState<AutoColorState>>,
) {
    timer.skip_tick = false;

    // Если одновременно нажаты цифра и пробел, приоритет у цифры.
    if let Some(index) = COLOR_KEYS
        .iter()
        .position(|key| keyboard.just_pressed(*key))
    {
        palette.current_index = index;
        timer.skip_tick = true;
    } else if keyboard.just_pressed(KeyCode::Space) {
        palette.next();
        timer.skip_tick = true;
    }

    if keyboard.just_pressed(KeyCode::KeyA) {
        next_state.set(match state.get() {
            AutoColorState::Off => AutoColorState::On,
            AutoColorState::On => AutoColorState::Off,
        });
        timer.skip_tick = true;
    }

    // После ручного действия новый цвет держится полную секунду.
    if timer.skip_tick {
        timer.timer.reset();
    }
}

/// При активном режиме продвигаем таймер на длительность кадра.
/// Один завершённый секундный интервал переводит палитру дальше.
fn auto_color_system(
    time: Res<Time>,
    mut timer: ResMut<AutoColorTimer>,
    mut palette: ResMut<ColorPalette>,
) {
    if timer.skip_tick {
        return;
    }
    if timer.timer.tick(time.delta()).just_finished() {
        palette.next();
    }
}

/// Компонент хранит ссылку; сам цвет лежит в Assets.
/// В Bevy 0.19 get_mut возвращает AssetMut, поэтому нужен mut.
fn apply_color_system(
    palette: Res<ColorPalette>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&MeshMaterial3d<StandardMaterial>, With<LabObject>>,
) {
    if !palette.is_changed() {
        return;
    }
    for handle in &query {
        if let Some(mut material) = materials.get_mut(handle.id()) {
            material.base_color = palette.current();
        }
    }
}

/// Угол за кадр = угловая скорость × длительность кадра.
/// rotate_y вращает вокруг мировой Y, поскольку у тора нет родителя.
fn rotation_system(time: Res<Time>, mut query: Query<&mut Transform, With<LabObject>>) {
    for mut transform in &mut query {
        transform.rotate_y(ROTATION_SPEED * time.delta_secs());
    }
}
