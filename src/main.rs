use bevy::prelude::*;

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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Lab 01 | Space: next | 1-5: color | A: auto".into(),
                resolution: (1000, 700).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.22, 0.88, 0.11)))
        .add_plugins(Lab01Plugin)
        .run();
}

#[derive(Component)]
struct LabObject;

#[derive(Resource)]
struct ColorPalette {
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

#[derive(Resource)]
struct AutoColorTimer {
    timer: Timer,
    skip_tick: bool,
}

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AutoColorState {
    #[default]
    Off,
    On,
}

struct Lab01Plugin;

impl Plugin for Lab01Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColorPalette>()
            .insert_resource(AutoColorTimer {
                timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                skip_tick: false,
            })
            .init_state::<AutoColorState>()
            .add_systems(Startup, setup_torus)
            .add_systems(
                Update,
                (
                    keyboard_input_system,
                    auto_color_system.run_if(in_state(AutoColorState::On)),
                    apply_color_system,
                )
                    .chain(),
            )
            .add_systems(Update, rotation_system);
    }
}

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
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.8, 2.0, 3.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_xyz(3.0, 5.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

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

fn rotation_system(time: Res<Time>, mut query: Query<&mut Transform, With<LabObject>>) {
    for mut transform in &mut query {
        transform.rotate_y(ROTATION_SPEED * time.delta_secs());
    }
}
