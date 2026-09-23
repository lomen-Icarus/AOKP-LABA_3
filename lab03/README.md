# bevy-lab03-scene-objects

Лабораторная работа №3: размещение графических объектов в составе сцены
(Rust / Bevy 0.19). Четыре тора разных цветов стоят на осях OX и OZ,
«носики» (конусы вдоль локальной +X) направлены в центр сцены.

## Запуск

```
cargo run --locked
```

Клавиша `R` включает и выключает обращение объектов вокруг центра
(дополнительное задание). При старте сцена статична и совпадает с примером.

## Проверка

```
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo run --locked --example capture   # кадры в screenshots/, под Linux без экрана: xvfb-run -a ...
```

## Структура

| Файл | Назначение |
|---|---|
| `src/graphic_object.rs` | Модуль GraphicObject: данные размещения, `to_transform`, `to_color`, порождение сущности, пересчёт Transform по изменению |
| `src/main.rs` | Главный файл: подключение модуля, окно, Lab03Plugin, список объектов, камера, свет, режим обращения |
| `src/tests.rs` | Автотесты геометрии и ECS-систем |
| `examples/capture.rs` | Получение кадров окна для отчёта |

## Требования к системе

Rust ≥ 1.95 (Bevy 0.19). Linux: X11 или XWayland и драйвер Vulkan
(для машины без GPU подходит Mesa lavapipe). Windows: ничего дополнительно.
