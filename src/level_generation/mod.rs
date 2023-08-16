pub mod generators;
pub mod map;

use bevy::prelude::*;

use crate::sprite_atlas::SpriteAtlas;

#[derive(Component)]
pub struct MapTile;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, display_map.run_if(resource_added::<map::Map>()));
    }
}

fn display_map(
    mut commands: Commands,
    map: ResMut<map::Map>,
    atlas: Res<SpriteAtlas>,
    old_tiles_query: Query<Entity, With<MapTile>>,
) {
    //clear old map
    for old_tile_entity in old_tiles_query.iter() {
        commands.entity(old_tile_entity).despawn_recursive();
    }

    for x in 0..map.width {
        for y in 0..map.height {
            let sprite_index = if let Some(tile) = map.get((x, y)) {
                tile.sprite_index
            } else {
                2499
            };
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: atlas.handle.clone(),
                    sprite: TextureAtlasSprite::new(sprite_index),
                    transform: Transform {
                        translation: Vec3::new(x as f32, y as f32, 0.0) * Vec3::splat(12.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                MapTile,
            ));
        }
    }
}
