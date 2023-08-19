use bevy::{prelude::*, utils::HashMap};
use rand::{thread_rng, Rng};

use crate::{
    level_generation::map::Map,
    position::{Position, PositionDelta},
    sprite_atlas::{SpriteAtlas, SpriteIndex},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum TurnState {
    #[default]
    Player,
    Enemy,
}
pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<TurnState>()
            .add_systems(PostStartup, (spawn_player, spawn_enemies))
            .add_systems(
                Update,
                player_movement.run_if(state_exists_and_equals(TurnState::Player)),
            )
            .add_systems(
                Update,
                enemy_movement.run_if(state_exists_and_equals(TurnState::Enemy)),
            )
            .add_systems(
                PostUpdate,
                (update_transform, center_camera_on_player).chain(),
            );
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
fn center_camera_on_player(
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
    mut player_query: Query<(&mut Movement, &mut Position), (With<Player>, Without<Enemy>)>,
    enemy_query: Query<&Position, (With<Enemy>, Without<Player>)>,
    map: Res<Map>,
    keyboard: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<TurnState>>,
) {
    let mut delta = PositionDelta::new(0, 0);
    if keyboard.just_pressed(KeyCode::W) {
        delta.y += 1;
    }
    if keyboard.just_pressed(KeyCode::A) {
        delta.x -= 1;
    }
    if keyboard.just_pressed(KeyCode::S) {
        delta.y -= 1;
    }
    if keyboard.just_pressed(KeyCode::D) {
        delta.x += 1;
    }

    if delta.x != 0 || delta.y != 0 {
        let (mut player_movement, mut player_position) = player_query.get_single_mut().unwrap();
        let new_position = *player_position + delta;

        //check that player won't collide with any enemies
        if enemy_query
            .iter()
            .any(|enemy_position| &new_position == enemy_position)
        {
            return;
        }

        if let Some(tile) = map.get(new_position.x, new_position.y) {
            if tile.passable {
                *player_position = new_position;
                player_movement.just_moved = true;
                next_state.set(TurnState::Enemy);
            }
        }
    }
}

fn enemy_movement(
    player_query: Query<&Position, (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<(Entity, &mut Position), (With<Enemy>, Without<Player>)>,
    map: Res<Map>,
    mut next_state: ResMut<NextState<TurnState>>,
) {
    let mut rng = thread_rng();

    let mut deltas = HashMap::new();
    let mut collides = HashMap::new();
    enemy_query.iter().for_each(|(entity, _)| {
        deltas.insert(
            entity,
            PositionDelta::new(rng.gen_range(-1..=1), rng.gen_range(-1..=1)),
        );
        collides.insert(entity, false);
    });

    //check for any collisions between enemies
    for [(entity1, pos1), (entity2, pos2)] in enemy_query.iter_combinations() {
        let new_pos1 = *pos1 + *deltas.get(&entity1).unwrap();

        let new_pos2 = *pos2 + *deltas.get(&entity2).unwrap();

        if new_pos1 == new_pos2 || *pos1 == new_pos2 || *pos2 == new_pos1 {
            collides.insert(entity1, true);
            collides.insert(entity2, true);
        }
    }

    //player collisions
    for (entity, position) in enemy_query.iter() {
        let new_position = *position + *deltas.get(&entity).unwrap();
        for player_position in player_query.iter() {
            if new_position == *player_position {
                collides.insert(entity, true);
            }
        }
    }

    for (entity, mut position) in enemy_query.iter_mut() {
        if !collides.get(&entity).unwrap() {
            let new_position = *position + *deltas.get(&entity).unwrap();
            if let Some(tile) = map.get(new_position.x, new_position.y) {
                if tile.passable {
                    *position = new_position;
                }
            }
        }
    }
    next_state.set(TurnState::Player);
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
        Position {
            x: map.player_spawn_points[0].0,
            y: map.player_spawn_points[0].1,
        },
    ));
}

fn update_transform(mut query: Query<(&mut Transform, &Position), Changed<Position>>) {
    for (mut trans, pos) in query.iter_mut() {
        trans.translation.x = pos.x as f32 * 12.0;
        trans.translation.y = pos.y as f32 * 12.0;
    }
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
            Position {
                x: point.0,
                y: point.1,
            },
        ));
    }
}
