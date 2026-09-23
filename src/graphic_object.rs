//! Модуль графического объекта: данные размещения, расчёт матрицы модели
//! (`Transform`) и порождение сущности на сцене.
//! Аналог пары GraphicObject.h / GraphicObject.cpp из исходной работы.

use bevy::asset::uuid_handle;
use bevy::prelude::*;

/// Внутренний и внешний радиусы тора-модели (аргументы `Torus::new`).
const TORUS_INNER_RADIUS: f32 = 0.4;
const TORUS_OUTER_RADIUS: f32 = 1.0;
/// Размеры «носика»: конус, вершина которого направлена вдоль локальной +X.
const NOSE_RADIUS: f32 = 0.22;
const NOSE_LENGTH: f32 = 0.6;

/// Модель в хранилище `Assets<Mesh>`. У мешей постоянные идентификаторы (UUID),
/// поэтому все объекты ссылаются на одну и ту же геометрию: меш создаётся
/// при первом вызове spawn_graphic_object, а следующие вызовы берут готовый.
const TORUS_MESH: Handle<Mesh> = uuid_handle!("6f1d3a52-3c1e-4d1b-9a7e-2b5c8e0f4a11");
const NOSE_MESH: Handle<Mesh> = uuid_handle!("a4c7e9b0-5d2f-4e83-8b16-7f3a9c2d6e54");

/// Регистрирует меш модели, если его ещё нет в хранилище.
fn ensure_mesh(meshes: &mut Assets<Mesh>, handle: &Handle<Mesh>, build: impl FnOnce() -> Mesh) {
    if !meshes.contains(handle) {
        meshes
            .insert(handle, build())
            .expect("для UUID-идентификатора вставка не завершается ошибкой");
    }
}

/// Положение носика в локальной системе координат модели.
/// Конус Bevy направлен вершиной вдоль +Y; поворот на −90° вокруг Z
/// укладывает его вершиной вдоль +X, сразу за внешним краем тора.
fn nose_transform() -> Transform {
    Transform::from_xyz(TORUS_OUTER_RADIUS + NOSE_LENGTH / 2.0, 0.0, 0.0)
        .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2))
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

    /// Размещение после обращения вокруг вертикальной оси сцены на `phase_deg`.
    /// Считается от исходного размещения, а не прибавляется к текущему,
    /// поэтому ошибки округления не накапливаются: радиус и направление
    /// носика на центр остаются точными при любой длительности работы.
    pub fn orbited(&self, phase_deg: f32) -> Self {
        Self {
            position: Quat::from_rotation_y(phase_deg.to_radians()) * self.position,
            angle_deg: (self.angle_deg - phase_deg).rem_euclid(360.0),
            color: self.color,
        }
    }
}

/// Система визуализации. Роль та же, что у update_visual_system
/// в лабораторной №2: перенести данные в то, что видит рендерер.
/// Отличие в условии запуска: здесь реагирует фильтр `Changed<GraphicObject>`
/// на изменение компонента, а не проверка изменения ресурса.
/// Аналог recalculateModelMatrix: при изменении позиции или угла
/// пересчитывается матрица модели (`Transform`). Материал переписывается
/// только когда цвет действительно другой: запись через `get_mut` помечает
/// ассет изменённым, и рендерер заново готовит его для GPU.
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
        let color = object.to_color();
        let color_changed = materials
            .get(material.id())
            .is_some_and(|current| current.base_color != color);
        if color_changed && let Some(mut current) = materials.get_mut(material.id()) {
            current.base_color = color;
        }
    }
}

/// Порождение сущности на сцене: аналог метода draw.
/// Регистрирует меш модели (один раз на все объекты) и материал объекта,
/// затем создаёт сущность с компонентами Mesh3d, MeshMaterial3d, Transform
/// и самим GraphicObject. Носик — дочерняя сущность с тем же материалом.
pub fn spawn_graphic_object(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    object: GraphicObject,
) {
    ensure_mesh(meshes, &TORUS_MESH, || {
        Torus::new(TORUS_INNER_RADIUS, TORUS_OUTER_RADIUS).into()
    });
    ensure_mesh(meshes, &NOSE_MESH, || {
        Cone::new(NOSE_RADIUS, NOSE_LENGTH).into()
    });
    let material = materials.add(StandardMaterial {
        base_color: object.to_color(),
        perceptual_roughness: 0.4,
        ..default()
    });
    println!(
        "Создан объект: {object:?}, носик {:?}, в центр: {}",
        object.nose_direction(),
        object.faces(Vec3::ZERO)
    );
    commands.spawn((
        Mesh3d(TORUS_MESH),
        MeshMaterial3d(material.clone()),
        object.to_transform(),
        object,
        children![(
            Mesh3d(NOSE_MESH),
            MeshMaterial3d(material),
            nose_transform()
        )],
    ));
}
