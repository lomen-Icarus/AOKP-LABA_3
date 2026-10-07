//! Лабораторная работа №3: размещение графических объектов в составе сцены.
use bevy::prelude::*;

use super::Lab;
use super::graphic_object::{GraphicObject, spawn_graphic_object};

#[cfg(test)]
mod tests;

const ORBIT_SPEED_DEG: f32 = 30.0;

#[derive(Resource, Default)]
pub struct Orbit {
    pub enabled: bool,
}

pub struct Lab03Plugin;

impl Plugin for Lab03Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Orbit>()
            .add_systems(OnEnter(Lab::Lab3), (reset_lab3, setup_scene).chain())
            .add_systems(OnExit(Lab::Lab3), despawn_objects)
            .add_systems(
                Update,
                (keyboard_system, orbit_system)
                    .chain()
                    .run_if(in_state(Lab::Lab3)),
            );
    }
}

pub fn scene_objects() -> Vec<GraphicObject> {
    vec![
        // Красный — справа (+X), угол 180°
        GraphicObject::new(Vec3::new(4.0, 0.0, 0.0), 180.0, Vec3::new(1.0, 0.0, 0.0)),
        // Синий — слева (−X), угол 0°
        GraphicObject::new(Vec3::new(-4.0, 0.0, 0.0), 0.0, Vec3::new(0.0, 0.0, 1.0)),
        // Зелёный — сзади (−Z), угол 90°
        GraphicObject::new(Vec3::new(0.0, 0.0, -4.0), 90.0, Vec3::new(0.0, 1.0, 0.0)),
        // Белый — спереди (+Z), угол 270°
        GraphicObject::new(Vec3::new(0.0, 0.0, 4.0), 270.0, Vec3::new(1.0, 1.0, 1.0)),
    ]
}

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
    let graphic_objects: Vec<GraphicObject> = scene_objects();
    for obj in graphic_objects {
        spawn_graphic_object(&mut commands, &mut meshes, &mut materials, obj);
    }

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

fn despawn_objects(mut commands: Commands, query: Query<Entity, With<GraphicObject>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

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

/// Доп. задание: обращение объектов вокруг центра сцены по клавише R.
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
