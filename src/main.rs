use bevy::prelude::*;
use mystery_dungeon::{
    actor::ActorPlugin,
    camera_controls::{CameraControlsPlugin, MainCamera},
    fov::FovPlugin,
    map_generation::{generators::MapGeneratorSettings, map::Map, MapPlugin},
    sprite_atlas::SpriteAtlasPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            SpriteAtlasPlugin,
            CameraControlsPlugin,
            FovPlugin,
            MapPlugin,
            ActorPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));

    let map_settings = MapGeneratorSettings::default();
    let map = Map::new(map_settings);
    eprintln!("Map Created!");
    commands.insert_resource(map_settings);
    commands.insert_resource(map);
}
