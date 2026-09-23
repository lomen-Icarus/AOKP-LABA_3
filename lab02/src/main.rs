use bevy::prelude::*;
use bevy_lab02_color_cycle::Lab02Plugin;

// Окно и базовые плагины движка; вся логика лабораторной — в Lab02Plugin.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Lab 02 | авто-смена цвета | Space: следующий | 1-5 | L: плавно".into(),
                resolution: (1000, 700).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.22, 0.88, 0.11)))
        .add_plugins(Lab02Plugin)
        .run();
}
