use bevy::prelude::*;

use crate::{
    map_generation::map::Map,
    sprite_atlas::{SpriteAtlas, SpriteIndex},
};

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_player)
            .add_systems(Update, player_movement);
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Actor {
    _health: f32,
}

// fn center_camera(
//     player_query: Query<&Transform, With<Player>>,
//     mut camera_query: Query<&mut Transform, (Without<Player>, With<Camera>)>,
// ) {
//     let player_transform = player_query.single();
//     let mut camera_transform = camera_query.single_mut();

//     camera_transform.translation.x = player_transform.translation.x;
//     camera_transform.translation.y = player_transform.translation.y;
// }

fn player_movement(
    mut player_query: Query<&mut Transform, With<Player>>,
    map: Res<Map>,
    keyboard: Res<Input<KeyCode>>,
) {
    let mut delta: [isize; 2] = [0, 0];
    if keyboard.just_pressed(KeyCode::W) {
        delta[1] += 1;
    }
    if keyboard.just_pressed(KeyCode::A) {
        delta[0] -= 1;
    }
    if keyboard.just_pressed(KeyCode::S) {
        delta[1] -= 1;
    }
    if keyboard.just_pressed(KeyCode::D) {
        delta[0] += 1;
    }

    if delta != [0, 0] {
        let mut player_transform = player_query.get_single_mut().unwrap();
        let new_index_vec: Vec<usize> = (0..2)
            .map(|i| {
                (player_transform.translation[i] as usize / 12).saturating_add_signed(delta[i])
            })
            .collect();
        let new_index = (new_index_vec[0], new_index_vec[1]);
        if let Some(tile) = map.get(new_index) {
            if tile.passable {
                player_transform.translation.x += delta[0] as f32 * 12.0;
                player_transform.translation.y += delta[1] as f32 * 12.0;
                eprintln!("Player Moved: ({:?}, {:?})", delta[0], delta[1]);
                eprintln!("Player Translation: {:?}", player_transform.translation);
            }
        }
    }
}

fn spawn_player(mut commands: Commands, atlas: Res<SpriteAtlas>, map: Res<Map>) {
    eprintln!("Player Created!");
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: atlas.handle.clone(),
            sprite: TextureAtlasSprite::new(SpriteIndex::Player as usize),
            transform: Transform {
                translation: Vec3::new(map.start.0 as f32, map.start.1 as f32, 1.0)
                    * Vec3::splat(12.0),
                ..Default::default()
            },
            ..Default::default()
        },
        Actor { _health: 100. },
        Player,
    ));
}
