//! Модуль графического объекта: данные размещения, расчёт матрицы модели
//! (`Transform`) и порождение сущности на сцене.
//! Аналог пары GraphicObject.h / GraphicObject.cpp из исходной работы.

use bevy::prelude::*;

/// Внутренний и внешний радиусы тора-модели (аргументы `Torus::new`).
const TORUS_INNER_RADIUS: f32 = 0.45;
const TORUS_OUTER_RADIUS: f32 = 1.0;
/// Размеры «носика»: конус, вершина которого направлена вдоль локальной +X.
const NOSE_RADIUS: f32 = 0.22;
const NOSE_LENGTH: f32 = 0.6;

/// Модель. Геометрия хранится в `Assets<Mesh>` один раз,
/// каждый графический объект получает только handle на неё.
#[derive(Resource, Clone)]
pub struct GraphicModel {
    pub body: Handle<Mesh>,
    pub nose: Handle<Mesh>,
}

impl GraphicModel {
    pub fn new(meshes: &mut Assets<Mesh>) -> Self {
        Self {
            body: meshes.add(Torus::new(TORUS_INNER_RADIUS, TORUS_OUTER_RADIUS)),
            nose: meshes.add(Cone::new(NOSE_RADIUS, NOSE_LENGTH)),
        }
    }

    /// Положение носика в локальной системе координат модели.
    /// Конус Bevy направлен вершиной вдоль +Y; поворот на −90° вокруг Z
    /// укладывает его вершиной вдоль +X, сразу за внешним краем тора.
    pub fn nose_transform() -> Transform {
        Transform::from_xyz(TORUS_OUTER_RADIUS + NOSE_LENGTH / 2.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2))
    }
}

/// Графический объект: параметры размещения модели в мировой системе координат.
/// Хранится как компонент сущности вместе с рассчитанным `Transform`.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct GraphicObject {
    /// Позиция в world space.
    pub position: Vec3,
    /// Угол поворота вокруг Oy в градусах (по часовой стрелке, как в оригинале).
    pub angle_deg: f32,
    /// Цвет (r, g, b) в диапазоне 0..=1.
    pub color: Vec3,
}

impl GraphicObject {
    pub const fn new(position: Vec3, angle_deg: f32, color: Vec3) -> Self {
        Self {
            position,
            angle_deg,
            color,
        }
    }

    /// Матрица модели. Аналог recalculateModelMatrix: перенос в `position`
    /// и поворот вокруг Oy. Отрицательный знак сохраняет направление
    /// «по часовой стрелке» из исходной работы.
    pub fn to_transform(&self) -> Transform {
        let angle_rad = -self.angle_deg.to_radians();
        Transform::from_translation(self.position).with_rotation(Quat::from_rotation_y(angle_rad))
    }

    pub fn to_color(&self) -> Color {
        Color::srgb(self.color.x, self.color.y, self.color.z)
    }

    /// Направление носика в мировой системе координат: локальная +X после поворота.
    pub fn nose_direction(&self) -> Vec3 {
        self.to_transform().rotation * Vec3::X
    }

    /// Смотрит ли носик в точку `target` (косинус угла между направлениями ≈ 1).
    pub fn faces(&self, target: Vec3) -> bool {
        let to_target = target - self.position;
        to_target.length_squared() > 0.0 && self.nose_direction().dot(to_target.normalize()) > 0.999
    }

    /// Поворот размещения вокруг вертикальной оси сцены на `delta_deg`.
    /// Позиция и носик поворачиваются вместе, поэтому ориентация
    /// на центр сохраняется.
    pub fn orbit_around_center(&mut self, delta_deg: f32) {
        self.position = Quat::from_rotation_y(delta_deg.to_radians()) * self.position;
        self.angle_deg = (self.angle_deg - delta_deg).rem_euclid(360.0);
    }
}

/// Система визуализации (как update_visual_system в лабораторной №2).
/// Аналог recalculateModelMatrix: когда позиция, угол или цвет объекта
/// изменились, матрица модели (`Transform`) и материал пересчитываются.
/// Фильтр `Changed` даёт то же, что вызов пересчёта из сеттеров в C++,
/// но без ручного отслеживания: Bevy сам помечает изменённые компоненты.
pub fn update_visual_system(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<
        (
            &GraphicObject,
            &mut Transform,
            &MeshMaterial3d<StandardMaterial>,
        ),
        Changed<GraphicObject>,
    >,
) {
    for (object, mut transform, material) in &mut query {
        *transform = object.to_transform();
        if let Some(mut material) = materials.get_mut(material.id()) {
            material.base_color = object.to_color();
        }
    }
}

/// Порождение сущности на сцене: аналог метода draw.
/// Тор и носик используют общие меши модели и один материал объекта.
pub fn spawn_graphic_object(
    commands: &mut Commands,
    model: &GraphicModel,
    materials: &mut Assets<StandardMaterial>,
    object: GraphicObject,
) -> Entity {
    let material = materials.add(StandardMaterial {
        base_color: object.to_color(),
        perceptual_roughness: 0.4,
        ..default()
    });
    let transform = object.to_transform();
    info!(
        "Создан объект: позиция {:?}, угол {:.0}°, цвет {:?}, носик направлен {:?}, в центр: {}",
        object.position,
        object.angle_deg,
        object.color,
        object.nose_direction(),
        object.faces(Vec3::ZERO)
    );
    commands
        .spawn((
            Mesh3d(model.body.clone()),
            MeshMaterial3d(material.clone()),
            transform,
            object,
            children![(
                Mesh3d(model.nose.clone()),
                MeshMaterial3d(material),
                GraphicModel::nose_transform(),
            )],
        ))
        .id()
}
