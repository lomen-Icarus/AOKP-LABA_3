//! Лабораторная работа №2: glam, контейнер Vec и автоматическая смена цвета.
//! Строится на лабораторной №1: та же сцена с тором, маркер, вращение и
//! ручной выбор цвета. Новое: палитра `Vec<Vec3>`, раздельные системы
//! симуляции и визуализации, смена цвета раз в секунду без участия пользователя.
//! В единой программе лаба активна, когда выбрана клавишей F2.

use bevy::prelude::*;

use super::Lab;

#[cfg(test)]
mod tests;

/// Угловая скорость вращения тора, радианы в секунду (из лабораторной №1).
pub const ROTATION_SPEED: f32 = 0.7;
/// Интервал автоматической смены цвета, секунды.
pub const COLOR_INTERVAL_SECS: f32 = 1.0;
/// Длительность плавного перехода между цветами (доп. задание 2), секунды.
pub const TRANSITION_SECS: f32 = 0.4;

/// Клавиши прямого выбора цвета: 1 выбирает индекс 0 и так далее.
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

/// Глобальные данные симуляции: список цветов и индекс текущего цвета.
/// Цвет хранится как `Vec3` из glam (аналог glm::vec3): x, y, z — каналы R, G, B.
#[derive(Resource, Debug, Clone)]
pub struct ColorPalette {
    /// Список цветов — контейнер `Vec` (аналог std::vector).
    pub colors: Vec<Vec3>,
    /// Индекс текущего цвета.
    pub current_index: usize,
    /// Индекс предыдущего цвета: начало плавного перехода.
    pub previous_index: usize,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorPalette {
    /// Палитра из задания: белый, синий, красный, жёлтый, фиолетовый.
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

    /// Текущий цвет как вектор glam.
    pub fn current_rgb(&self) -> Vec3 {
        self.colors[self.current_index]
    }

    /// Текущий цвет в типе рендерера Bevy.
    pub fn current_color(&self) -> Color {
        rgb_to_color(self.current_rgb())
    }

    /// Переход к следующему цвету по кругу: после последнего снова первый.
    /// Длина берётся из самого `Vec`, поэтому новые цвета добавляются без правок.
    pub fn next_color(&mut self) -> Color {
        self.select((self.current_index + 1) % self.colors.len());
        self.current_color()
    }

    /// Прямой выбор цвета по индексу. Возвращает `false` для индекса вне списка.
    pub fn select(&mut self, index: usize) -> bool {
        if index >= self.colors.len() {
            return false;
        }
        self.previous_index = self.current_index;
        self.current_index = index;
        true
    }

    /// Промежуточный цвет перехода: линейная интерполяция компонент `Vec3`.
    /// `t = 0` — предыдущий цвет, `t = 1` — текущий.
    pub fn blended_rgb(&self, t: f32) -> Vec3 {
        let from = self.colors[self.previous_index];
        from.lerp(self.current_rgb(), t.clamp(0.0, 1.0))
    }
}

/// Преобразование математического вектора в цвет sRGB.
pub fn rgb_to_color(rgb: Vec3) -> Color {
    Color::srgb(rgb.x, rgb.y, rgb.z)
}

/// Повторяющийся секундный таймер автоматической смены цвета.
#[derive(Resource)]
pub struct AutoColorTimer(pub Timer);

/// Настройки визуализации: мгновенная смена или плавный переход.
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
            // При каждом входе в лабу 2 сцена и данные создаются заново.
            .add_systems(
                OnEnter(Lab::Lab2),
                (reset_lab2, glam_demo_system, setup_scene).chain(),
            )
            // Порядок: ввод пользователя, затем симуляция, затем визуализация,
            // чтобы новый цвет попал на экран в том же кадре.
            .add_systems(
                Update,
                (keyboard_system, simulation_system, update_visual_system)
                    .chain()
                    .run_if(in_state(Lab::Lab2)),
            )
            .add_systems(Update, rotation_system.run_if(in_state(Lab::Lab2)));
    }
}

/// Демонстрация основных операций glam; результат выводится в консоль при запуске.
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

/// Начальное состояние лабы 2: белый цвет, таймер с нуля, зелёный фон.
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

/// Начальная сцена из лабораторной №1: наклонённый тор, камера и свет.
/// Материал получает первый цвет палитры, чтобы индекс и экран совпадали.
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
        // Наклон делает вращение вокруг мировой Y заметным.
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

/// Действия пользователя (доп. задание 1 и выбор из лабораторной №1).
/// Автоматический режим при этом не выключается: ручной выбор только
/// перезапускает секундный интервал, чтобы новый цвет держался полную секунду.
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

/// Система симуляции: вызывается каждый кадр и меняет только глобальные данные.
/// Таймер накапливает реальное время кадров; по завершении интервала
/// индекс палитры переходит к следующему цвету.
pub fn simulation_system(
    time: Res<Time>,
    mut timer: ResMut<AutoColorTimer>,
    mut palette: ResMut<ColorPalette>,
) {
    // tick() добавляет длительность кадра; just_finished() истинно ровно
    // в том кадре, где накопилась полная секунда.
    if timer.0.tick(time.delta()).just_finished() {
        palette.next_color();
        print_state("simulation", &palette);
    }
}

/// Система визуализации: только читает палитру и переносит цвет в материал.
/// В мгновенном режиме работает лишь при изменении палитры, в плавном —
/// каждый кадр до конца перехода.
pub fn update_visual_system(
    palette: Res<ColorPalette>,
    timer: Res<AutoColorTimer>,
    settings: Res<VisualSettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&MeshMaterial3d<StandardMaterial>, With<LabObject>>,
    mut was_transition: Local<bool>,
) {
    let t = timer.0.elapsed_secs() / TRANSITION_SECS;
    let transition = settings.smooth && t < 1.0;
    // Кадр после окончания перехода тоже записывается: цвет доходит до точного значения.
    if !(transition || *was_transition || palette.is_changed() || settings.is_changed()) {
        return;
    }
    *was_transition = transition;
    let rgb = if transition {
        palette.blended_rgb(t)
    } else {
        palette.current_rgb()
    };
    for handle in &query {
        if let Some(mut material) = materials.get_mut(handle.id()) {
            material.base_color = rgb_to_color(rgb);
        }
    }
}

/// Вращение из лабораторной №1: угол за кадр = скорость × длительность кадра.
pub fn rotation_system(time: Res<Time>, mut query: Query<&mut Transform, With<LabObject>>) {
    for mut transform in &mut query {
        transform.rotate_y(ROTATION_SPEED * time.delta_secs());
    }
}

/// Отладочный вывод состояния глобальных данных: индекс и RGB текущего цвета.
fn print_state(source: &str, palette: &ColorPalette) {
    let c = palette.current_rgb();
    println!(
        "[{source}] индекс = {}, RGB = ({:.2}, {:.2}, {:.2})",
        palette.current_index, c.x, c.y, c.z
    );
}
