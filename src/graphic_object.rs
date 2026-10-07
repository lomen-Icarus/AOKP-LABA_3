//! Модуль графического объекта (аналог GraphicObject.h / GraphicObject.cpp).
use bevy::prelude::*;

const TORUS_INNER_RADIUS: f32 = 0.4;
const TORUS_OUTER_RADIUS: f32 = 1.0;
/// Носик — конус вдоль +X: без него поворот тора не виден.
const NOSE_RADIUS: f32 = 0.22;
const NOSE_LENGTH: f32 = 0.6;

#[derive(Component, Debug, Clone)]
pub struct GraphicObject {
    pub position: Vec3, // Позиция объекта в глобальной системе координат (world space)
    pub angle_deg: f32, // Угол поворота вокруг оси Oy (в градусах)
    pub color: Vec3,    // Цвет модели в формате (r, g, b) в диапазоне [0.0, 1.0]
}

impl GraphicObject {
    /// Конструктор
    pub fn new(position: Vec3, angle_deg: f32, color: Vec3) -> Self {
        Self {
            position,
            angle_deg,
            color,
        }
    }

    /// Возвращает готовый компонент Transform.
    pub fn to_transform(&self) -> Transform {
        // Перевод градусов в радианы с инверсией знака
        let angle_rad = -self.angle_deg.to_radians();
        // Сборка Transform из переноса и поворота
        Transform::from_translation(self.position).with_rotation(Quat::from_rotation_y(angle_rad))
    }

    /// Преобразование вектора цвета в тип Color движка Bevy
    pub fn to_color(&self) -> Color {
        Color::srgb(self.color.x, self.color.y, self.color.z)
    }
}

pub fn spawn_graphic_object(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    object: GraphicObject,
) {
    println!("Создан объект: {:?}", object);
    let material = materials.add(StandardMaterial {
        base_color: object.to_color(),
        ..default()
    });
    let nose = (
        Mesh3d(meshes.add(Cone::new(NOSE_RADIUS, NOSE_LENGTH))),
        MeshMaterial3d(material.clone()),
        Transform::from_xyz(TORUS_OUTER_RADIUS + NOSE_LENGTH / 2.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
    );
    commands.spawn((
        Mesh3d(meshes.add(Torus::new(TORUS_INNER_RADIUS, TORUS_OUTER_RADIUS))),
        MeshMaterial3d(material),
        object.to_transform(),
        object,
        children![nose],
    ));
}
