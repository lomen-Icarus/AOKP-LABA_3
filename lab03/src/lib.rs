//! Лабораторная работа №3: размещение графических объектов в составе сцены.
//! Здесь находятся плагин, список размещений и системы; окно создаёт main.rs.

use bevy::prelude::*;

pub mod graphic_object;
#[cfg(test)]
mod tests;

use graphic_object::{GraphicModel, GraphicObject, spawn_graphic_object, sync_graphic_objects};

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
        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(GlobalAmbientLight {
                color: Color::WHITE,
                brightness: 250.0,
                affects_lightmapped_meshes: true,
            })
            .init_resource::<Orbit>()
            .add_systems(Startup, setup_scene)
            .add_systems(
                Update,
                (toggle_orbit_system, orbit_system, sync_graphic_objects).chain(),
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

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Модель создаётся один раз; объекты ссылаются на неё по handle.
    let model = GraphicModel::new(&mut meshes);
    let graphic_objects: Vec<GraphicObject> = scene_objects();
    for obj in graphic_objects {
        spawn_graphic_object(&mut commands, &model, &mut materials, obj);
    }
    commands.insert_resource(model);

    // Наблюдатель: положение камеры из исходной работы, взгляд в начало координат.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(10.0, 15.0, 17.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn toggle_orbit_system(keyboard: Res<ButtonInput<KeyCode>>, mut orbit: ResMut<Orbit>) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        orbit.enabled = !orbit.enabled;
        info!(
            "Обращение вокруг центра: {}",
            if orbit.enabled {
                "включено"
            } else {
                "выключено"
            }
        );
    }
}

/// Дополнительное задание: параметры размещения меняются каждый кадр,
/// а `Transform` заново рассчитывает система sync_graphic_objects.
fn orbit_system(time: Res<Time>, orbit: Res<Orbit>, mut query: Query<&mut GraphicObject>) {
    if !orbit.enabled {
        return;
    }
    let delta_deg = ORBIT_SPEED_DEG * time.delta_secs();
    for mut object in &mut query {
        object.orbit_around_center(delta_deg);
    }
}
