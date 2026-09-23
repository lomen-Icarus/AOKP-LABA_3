//! Модуль графического объекта: данные размещения, расчёт матрицы модели
//! (`Transform`), цвет и порождение сущности на сцене.
//! Аналог пары GraphicObject.h / GraphicObject.cpp из исходной работы.

use bevy::prelude::*;

/// Внутренний и внешний радиусы тора (аргументы `Torus::new`), как в лабах 1 и 2.
const TORUS_INNER_RADIUS: f32 = 0.4;
const TORUS_OUTER_RADIUS: f32 = 1.0;
/// Размеры «носика» — небольшого конуса вдоль локальной оси +X.
/// Тор симметричен относительно Oy, без носика его поворот не виден.
const NOSE_RADIUS: f32 = 0.22;
const NOSE_LENGTH: f32 = 0.6;

/// Графический объект: параметры размещения модели в мировой системе координат.
#[derive(Component, Debug, Clone)]
pub struct GraphicObject {
    pub position: Vec3, // Позиция объекта в глобальной системе координат (world space)
    pub angle_deg: f32, // Угол поворота вокруг оси Oy (в градусах)
    pub color: Vec3,    // Цвет модели в формате (r, g, b) в диапазоне [0.0, 1.0]
}

impl GraphicObject {
    /// Конструктор.
    pub fn new(position: Vec3, angle_deg: f32, color: Vec3) -> Self {
        Self {
            position,
            angle_deg,
            color,
        }
    }

    /// Матрица модели (аналог recalculateModelMatrix): перенос в `position`
    /// и поворот вокруг Oy. Минус нужен, чтобы поворот шёл по часовой
    /// стрелке, если смотреть сверху, как в исходной работе.
    pub fn to_transform(&self) -> Transform {
        let angle_rad = -self.angle_deg.to_radians();
        Transform::from_translation(self.position).with_rotation(Quat::from_rotation_y(angle_rad))
    }

    /// Преобразование вектора цвета в тип Color движка Bevy.
    pub fn to_color(&self) -> Color {
        Color::srgb(self.color.x, self.color.y, self.color.z)
    }
}

/// Порождение сущности на сцене (аналог метода draw): регистрирует меш
/// и материал, затем создаёт сущность с компонентами Mesh3d, MeshMaterial3d,
/// Transform и самим GraphicObject. Носик — дочерняя сущность, поэтому
/// поворачивается вместе с тором.
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
    // Конус Bevy направлен вершиной вдоль +Y; поворот на −90° вокруг Z
    // укладывает его вдоль +X, сразу за внешним краем тора.
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
