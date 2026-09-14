use bevy::prelude::*;
use bevy_lab03_scene_objects::Lab03Plugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Lab 03 | Размещение объектов на сцене | R: вращение вокруг центра".into(),
                resolution: (1000, 700).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(Lab03Plugin)
        .run();
}
