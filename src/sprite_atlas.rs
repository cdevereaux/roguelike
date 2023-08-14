use bevy::prelude::*;

#[derive(Resource)]
pub struct SpriteAtlas {
    pub handle: Handle<TextureAtlas>,
}

pub struct SpriteAtlasPlugin;

impl Plugin for SpriteAtlasPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("urizen_onebit_tileset__v1d0_transparent.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(12.0, 12.0),
        103,
        50,
        Some(Vec2::new(1.0, 1.0)),
        Some(Vec2::new(1.0, 1.0)),
    );
    let handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(SpriteAtlas {
        handle: handle.clone(),
    });
}


pub enum SpriteIndex {
    Player = 1648,
}