use bevy::{prelude::*, render::camera::RenderTarget, window::WindowRef};
use bevy_egui::{
    egui::{self, Ui},
    EguiContexts, EguiPlugin,
};
use mystery_dungeon::map_generation::map::Map;
use mystery_dungeon::{
    camera_controls::{CameraControlsPlugin, MainCamera},
    map_generation::{generators::MapGeneratorSettings, MapPlugin},
    sprite_atlas::SpriteAtlasPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            EguiPlugin,
            CameraControlsPlugin,
            MapPlugin,
            SpriteAtlasPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, (generation_options_ui,))
        .run();
}

#[derive(Component)]
pub struct ToolsWindow;

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));

    let map_settings = MapGeneratorSettings::default();
    let map = Map::new(map_settings);
    commands.insert_resource(map_settings);
    commands.insert_resource(map);

    // Spawn second window
    let tools_window = commands
        .spawn((
            Window {
                title: "Tools window".to_owned(),
                ..default()
            },
            ToolsWindow,
        ))
        .id();

    // Spawn dummy camera for window
    commands.spawn(Camera2dBundle {
        camera: Camera {
            target: RenderTarget::Window(WindowRef::Entity(tools_window)),
            ..default()
        },
        ..default()
    });
}

fn create_egui_drag_value(ui: &mut Ui, name: &str, value: &mut usize) {
    if ui.add(egui::DragValue::new(value).prefix(name)).changed() {
        *value = (*value).clamp(1, 500);
    }
}

fn generation_options_ui(
    mut contexts: EguiContexts,
    mut map: ResMut<Map>,
    window_query: Query<Entity, With<ToolsWindow>>,
    mut settings: ResMut<MapGeneratorSettings>,
) {
    egui::CentralPanel::default().show(
        contexts.ctx_for_window_mut(window_query.get_single().unwrap()),
        |ui| {
            use MapGeneratorSettings::*;
            match &mut *settings {
                Cavern(settings) => {
                    create_egui_drag_value(ui, "Cavern Count: ", &mut settings.cavern_count);
                    create_egui_drag_value(ui, "Cavern Distance: ", &mut settings.max_cavern_dist);
                    create_egui_drag_value(ui, "Walk Count: ", &mut settings.walk_count);
                    create_egui_drag_value(ui, "Walk Length: ", &mut settings.walk_len);
                }
            }
            if ui.button("Reset").clicked() {
                map.reset();
                map.generate(*settings);
            }
        },
    );
}
