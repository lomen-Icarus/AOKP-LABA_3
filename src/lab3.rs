//! Лабораторная работа №3: размещение графических объектов в составе сцены.
//! Список объектов `Vec<GraphicObject>`, камера, свет и системы сцены.
//! Логика графического объекта — в модуле graphic_object.rs.
//! От лабораторных №1 и №2 взяты модель-тор, цвет в виде `Vec3`,
//! контейнер `Vec` и порядок систем через `.chain()`.
//! В единой программе лаба активна, когда выбрана клавишей F3.

use bevy::prelude::*;

use super::Lab;
use super::graphic_object::{GraphicObject, spawn_graphic_object};

#[cfg(test)]
mod tests;

/// Скорость обращения объектов вокруг центра сцены (дополнительное задание),
/// градусы в секунду.
const ORBIT_SPEED_DEG: f32 = 30.0;

/// Режим обращения объектов вокруг центра сцены. Выключен при запуске,
/// чтобы кадр совпадал с примером из задания.
#[derive(Resource, Default)]
pub struct Orbit {
    pub enabled: bool,
}

pub struct Lab03Plugin;

impl Plugin for Lab03Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Orbit>()
            // При каждом входе в лабу 3 сцена создаётся заново,
            // при выходе объекты удаляются.
            .add_systems(OnEnter(Lab::Lab3), (reset_lab3, setup_scene).chain())
            .add_systems(OnExit(Lab::Lab3), despawn_objects)
            // Сначала ввод, затем движение, как в лабораторной №2.
            .add_systems(
                Update,
                (keyboard_system, orbit_system)
                    .chain()
                    .run_if(in_state(Lab::Lab3)),
            );
    }
}

/// Список размещений: аналог std::vector<GraphicObject> из исходной работы.
pub fn scene_objects() -> Vec<GraphicObject> {
    vec![
        // Красный — справа (+X), носик к центру: угол 180°.
        GraphicObject::new(Vec3::new(4.0, 0.0, 0.0), 180.0, Vec3::new(1.0, 0.0, 0.0)),
        // Синий — слева (−X), угол 0°.
        GraphicObject::new(Vec3::new(-4.0, 0.0, 0.0), 0.0, Vec3::new(0.0, 0.0, 1.0)),
        // Зелёный — сзади (−Z), угол 90°.
        GraphicObject::new(Vec3::new(0.0, 0.0, -4.0), 90.0, Vec3::new(0.0, 1.0, 0.0)),
        // Белый — спереди (+Z), угол 270°.
        GraphicObject::new(Vec3::new(0.0, 0.0, 4.0), 270.0, Vec3::new(1.0, 1.0, 1.0)),
    ]
}

/// Начальное состояние лабы 3: обращение выключено, фон чёрный,
/// рассеянный свет ярче, чтобы тёмные цвета торов были видны.
fn reset_lab3(mut commands: Commands) {
    commands.insert_resource(Orbit::default());
    commands.insert_resource(ClearColor(Color::BLACK));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 250.0,
        affects_lightmapped_meshes: true,
    });
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Динамический массив параметров объектов (аналог std::vector).
    let graphic_objects: Vec<GraphicObject> = scene_objects();
    for obj in graphic_objects {
        spawn_graphic_object(&mut commands, &mut meshes, &mut materials, obj);
    }

    // Наблюдатель: положение камеры из исходной работы, взгляд в начало координат.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(10.0, 15.0, 17.5).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(Lab::Lab3),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(Lab::Lab3),
    ));
}

/// Удаляет объекты лабы 3 при переходе к другой лабе.
fn despawn_objects(mut commands: Commands, query: Query<Entity, With<GraphicObject>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Ввод пользователя: клавиша R включает и выключает обращение вокруг центра.
pub fn keyboard_system(keyboard: Res<ButtonInput<KeyCode>>, mut orbit: ResMut<Orbit>) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        orbit.enabled = !orbit.enabled;
        println!(
            "[keyboard] обращение вокруг центра: {}",
            if orbit.enabled {
                "включено"
            } else {
                "выключено"
            }
        );
    }
}

/// Дополнительное задание: обращение всех объектов вокруг центра сцены.
/// `rotate_around` поворачивает и позицию, и ориентацию объекта,
/// поэтому носики остаются направлены в центр.
pub fn orbit_system(
    time: Res<Time>,
    orbit: Res<Orbit>,
    mut query: Query<&mut Transform, With<GraphicObject>>,
) {
    if !orbit.enabled {
        return;
    }
    let angle = (ORBIT_SPEED_DEG * time.delta_secs()).to_radians();
    for mut transform in &mut query {
        transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(angle));
    }
}
