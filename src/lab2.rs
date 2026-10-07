//! Лабораторная работа №2: glam, контейнер Vec и автоматическая смена цвета.
use bevy::prelude::*;

use super::Lab;

#[cfg(test)]
mod tests;

pub const ROTATION_SPEED: f32 = 0.7;
pub const COLOR_INTERVAL_SECS: f32 = 1.0;
pub const TRANSITION_SECS: f32 = 0.4;

const COLOR_KEYS: [KeyCode; 5] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
];

#[derive(Component)]
pub struct LabObject;

#[derive(Resource, Debug, Clone)]
pub struct ColorPalette {
    /// Список цветов. Тип Vec3 — аналог glm::vec3.
    /// Компоненты (x, y, z) кодируют каналы (Red, Green, Blue).
    pub colors: Vec<Vec3>,
    /// Индекс текущего выбранного цвета в массиве.
    pub current_index: usize,
    pub previous_index: usize,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorPalette {
    /// Конструктор палитры: инициализирует массив 5 базовыми цветами
    pub fn new() -> Self {
        Self {
            colors: vec![
                Vec3::new(1.0, 1.0, 1.0), // белый
                Vec3::new(0.0, 0.0, 1.0), // синий
                Vec3::new(1.0, 0.0, 0.0), // красный
                Vec3::new(1.0, 1.0, 0.0), // жёлтый
                Vec3::new(0.5, 0.0, 0.5), // фиолетовый
            ],
            current_index: 0,
            previous_index: 0,
        }
    }

    pub fn current_rgb(&self) -> Vec3 {
        self.colors[self.current_index]
    }

    /// Преобразует математический вектор Vec3 текущего цвета
    /// в графический тип Color пространства sRGB, понятный рендереру Bevy.
    pub fn current_color(&self) -> Color {
        rgb_to_color(self.current_rgb())
    }

    /// Вычисляет индекс следующего цвета по циклу и возвращает его значение.
    pub fn next_color(&mut self) -> Color {
        self.select((self.current_index + 1) % self.colors.len());
        self.current_color()
    }

    pub fn select(&mut self, index: usize) -> bool {
        if index >= self.colors.len() {
            return false;
        }
        self.previous_index = self.current_index;
        self.current_index = index;
        true
    }

    /// Плавный переход (доп. задание 2): интерполяция компонент Vec3.
    pub fn blended_rgb(&self, t: f32) -> Vec3 {
        let from = self.colors[self.previous_index];
        from.lerp(self.current_rgb(), t.clamp(0.0, 1.0))
    }
}

pub fn rgb_to_color(rgb: Vec3) -> Color {
    Color::srgb(rgb.x, rgb.y, rgb.z)
}

/// Ресурс-обёртка над таймером Bevy.
#[derive(Resource)]
pub struct AutoColorTimer(pub Timer);

#[derive(Resource, Default)]
pub struct VisualSettings {
    pub smooth: bool,
}

pub struct Lab02Plugin;

impl Plugin for Lab02Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ColorPalette::new())
            .insert_resource(AutoColorTimer(Timer::from_seconds(
                COLOR_INTERVAL_SECS,
                TimerMode::Repeating,
            )))
            .init_resource::<VisualSettings>()
            .add_systems(
                OnEnter(Lab::Lab2),
                (reset_lab2, glam_demo_system, setup_scene).chain(),
            )
            .add_systems(
                Update,
                (keyboard_system, simulation_system, update_visual_system)
                    .chain()
                    .run_if(in_state(Lab::Lab2)),
            )
            .add_systems(Update, rotation_system.run_if(in_state(Lab::Lab2)));
    }
}

pub fn glam_demo_system() {
    let a = Vec3::new(2.0, 0.0, 0.0);
    let b = Vec3::new(0.0, 3.0, 0.0);
    println!("[glam] a = {a}, b = {b}");
    println!(
        "[glam] a + b = {}, a - b = {}, a * 2 = {}",
        a + b,
        a - b,
        a * 2.0
    );
    println!(
        "[glam] normalize(a) = {}, |a| = {}, distance(a, b) = {:.4}",
        a.normalize(),
        a.length(),
        a.distance(b)
    );
    println!(
        "[glam] dot(a, b) = {}, cross(a, b) = {}",
        a.dot(b),
        a.cross(b)
    );
    let m = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let restored = m.inverse() * m;
    println!(
        "[glam] M^-1 * M = E: {}, M^T строка 4 = {}",
        restored.abs_diff_eq(Mat4::IDENTITY, 1e-6),
        m.transpose().row(3)
    );
}

fn reset_lab2(mut commands: Commands) {
    commands.insert_resource(ColorPalette::new());
    commands.insert_resource(AutoColorTimer(Timer::from_seconds(
        COLOR_INTERVAL_SECS,
        TimerMode::Repeating,
    )));
    commands.insert_resource(VisualSettings::default());
    commands.insert_resource(ClearColor(Color::srgb(0.22, 0.88, 0.11)));
    commands.insert_resource(GlobalAmbientLight::default());
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<ColorPalette>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Torus::new(0.4, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: palette.current_color(),
            perceptual_roughness: 0.35,
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(0.65)),
        LabObject,
        DespawnOnExit(Lab::Lab2),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.8, 2.0, 3.5).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(Lab::Lab2),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_xyz(3.0, 5.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(Lab::Lab2),
    ));
}

/// Пробел и цифры 1–5 меняют цвет вручную (доп. задание 1), L включает плавный переход.
pub fn keyboard_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut palette: ResMut<ColorPalette>,
    mut timer: ResMut<AutoColorTimer>,
    mut settings: ResMut<VisualSettings>,
) {
    let selected = COLOR_KEYS
        .iter()
        .position(|key| keyboard.just_pressed(*key));
    let manual = if let Some(index) = selected {
        palette.select(index)
    } else if keyboard.just_pressed(KeyCode::Space) {
        palette.next_color();
        true
    } else {
        false
    };
    if manual {
        timer.0.reset();
        print_state("keyboard", &palette);
    }
    if keyboard.just_pressed(KeyCode::KeyL) {
        settings.smooth = !settings.smooth;
        println!(
            "[keyboard] плавный переход: {}",
            if settings.smooth {
                "включён"
            } else {
                "выключен"
            }
        );
    }
}

pub fn simulation_system(
    time: Res<Time>,
    mut timer: ResMut<AutoColorTimer>,
    mut palette: ResMut<ColorPalette>,
) {
    // tick() учитывает реальное прошедшее время
    if timer.0.tick(time.delta()).just_finished() {
        palette.next_color();
        print_state("simulation", &palette);
    }
}

pub fn update_visual_system(
    palette: Res<ColorPalette>,
    timer: Res<AutoColorTimer>,
    settings: Res<VisualSettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&MeshMaterial3d<StandardMaterial>, With<LabObject>>,
) {
    let rgb = if settings.smooth {
        let t = timer.0.elapsed_secs() / TRANSITION_SECS;
        palette.blended_rgb(t)
    } else if palette.is_changed() || settings.is_changed() {
        palette.current_rgb()
    } else {
        return;
    };
    for handle in &query {
        if let Some(mut material) = materials.get_mut(handle.id()) {
            material.base_color = rgb_to_color(rgb);
        }
    }
}

pub fn rotation_system(time: Res<Time>, mut query: Query<&mut Transform, With<LabObject>>) {
    for mut transform in &mut query {
        transform.rotate_y(ROTATION_SPEED * time.delta_secs());
    }
}

fn print_state(source: &str, palette: &ColorPalette) {
    let c = palette.current_rgb();
    println!(
        "[{source}] индекс = {}, RGB = ({:.2}, {:.2}, {:.2})",
        palette.current_index, c.x, c.y, c.z
    );
}
