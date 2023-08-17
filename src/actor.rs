use bevy::prelude::*;

use crate::{
    level_generation::map::Map,
    sprite_atlas::{SpriteAtlas, SpriteIndex},
};

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, (spawn_player, spawn_enemies))
            .add_systems(Update, (player_movement, center_camera).chain());
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Actor {
    _health: f32,
}

#[derive(Component)]
pub struct Movement {
    just_moved: bool,
}

#[allow(clippy::type_complexity)]
fn center_camera(
    mut player_query: Query<(&mut Movement, &Transform), (With<Player>, Changed<Movement>)>,
    mut camera_query: Query<&mut Transform, (Without<Player>, With<Camera>)>,
) {
    if let Ok((mut player_movement, player_transform)) = player_query.get_single_mut() {
        if player_movement.just_moved {
            let mut camera_transform = camera_query.single_mut();
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.y = player_transform.translation.y;
            player_movement.just_moved = false;
        }
    }
}

fn player_movement(
    mut player_query: Query<(&mut Movement, &mut Transform), With<Player>>,
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
        let (mut player_movement, mut player_transform) = player_query.get_single_mut().unwrap();
        let new_index_vec: Vec<usize> = (0..2)
            .map(|i| {
                (player_transform.translation[i] as usize / 12).saturating_add_signed(delta[i])
            })
            .collect();
        if let Some(tile) = map.get(new_index_vec[0], new_index_vec[1]) {
            if tile.passable {
                player_transform.translation.x += delta[0] as f32 * 12.0;
                player_transform.translation.y += delta[1] as f32 * 12.0;
                eprintln!("Player Moved: ({:?}, {:?})", delta[0], delta[1]);
                eprintln!("Player Translation: {:?}", player_transform.translation);
                player_movement.just_moved = true;
            }
        }
    }
}

fn spawn_player(mut commands: Commands, atlas: Res<SpriteAtlas>, map: Res<Map>) {
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: atlas.handle.clone(),
            sprite: TextureAtlasSprite::new(SpriteIndex::Player as usize),
            transform: Transform {
                translation: Vec3::new(
                    map.player_spawn_points[0].0 as f32,
                    map.player_spawn_points[0].1 as f32,
                    1.0,
                ) * Vec3::splat(12.0),
                ..Default::default()
            },
            ..Default::default()
        },
        Actor { _health: 100. },
        Player,
        Movement { just_moved: false },
    ));
}

fn spawn_enemies(mut commands: Commands, atlas: Res<SpriteAtlas>, map: Res<Map>) {
    for point in &map.enemy_spawn_points {
        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: atlas.handle.clone(),
                sprite: TextureAtlasSprite::new(SpriteIndex::Bat as usize),
                transform: Transform {
                    translation: Vec3::new(point.0 as f32, point.1 as f32, 1.0) * Vec3::splat(12.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            Actor { _health: 10. },
            Enemy,
            Movement { just_moved: false },
        ));
    }
}
