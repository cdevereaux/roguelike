use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::{
    level_generation::map::{Map, ViewStatus},
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
                clear_moved_markers.run_if(state_changed::<TurnState>()),
            )
            .add_systems(
                Update,
                player_movement.run_if(state_exists_and_equals(TurnState::Player)),
            )
            .add_systems(Update, update_dormant_enemies)
            .add_systems(
                Update,
                (select_next_enemy_to_move, enemy_movement)
                    .chain()
                    .run_if(state_exists_and_equals(TurnState::Enemy)),
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
pub struct Dormant;

#[derive(Component)]
pub struct SelectedToMove;

#[derive(Component)]
pub struct MovedThisTurn;

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

fn select_next_enemy_to_move(
    mut commands: Commands,
    selection_query: Query<(), With<SelectedToMove>>,
    enemy_query: Query<Entity, (With<Enemy>, Without<MovedThisTurn>, Without<Dormant>)>,
    mut next_state: ResMut<NextState<TurnState>>,
) {
    if !selection_query.is_empty() {
        return;
    }
    for entity in enemy_query.iter() {
        commands.entity(entity).insert(SelectedToMove);
        return;
    }
    next_state.set(TurnState::Player);
}

fn enemy_movement(
    mut commands: Commands,
    mut target_query: Query<(Entity, &mut Position), With<SelectedToMove>>,
    actor_query: Query<&Position, (With<Actor>, Without<SelectedToMove>)>,
    map: Res<Map>,
) {
    if target_query.is_empty() {
        return;
    }
    let (entity, mut current_position) = target_query.single_mut();
    commands
        .entity(entity)
        .remove::<SelectedToMove>()
        .insert(MovedThisTurn);

    let mut rng = thread_rng();

    //TODO: Call function to decide delta
    let delta = PositionDelta::new(rng.gen_range(-1..=1), rng.gen_range(-1..=1));
    let new_position = *current_position + delta;

    //check for any collisions with actors
    if actor_query
        .iter()
        .all(|actor_position| *actor_position != new_position)
    {
        if let Some(tile) = map.get(new_position.x, new_position.y) {
            if tile.passable {
                *current_position = new_position;
            }
        }
    }
}

fn clear_moved_markers(mut commands: Commands, query: Query<Entity, With<MovedThisTurn>>) {
    query.iter().for_each(|entity| {
        commands.entity(entity).remove::<MovedThisTurn>();
    });
}

fn update_dormant_enemies(
    mut commands: Commands,
    query: Query<(Entity, &Position), With<Dormant>>,
    map: Res<Map>,
) {
    query.iter().for_each(|(entity, position)| {
        if let Some(tile) = map.get(position.x, position.y) {
            if tile.view_status != ViewStatus::Unexplored {
                commands.entity(entity).remove::<Dormant>();
            }
        }
    })
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
            Dormant,
            Movement { just_moved: false },
            Position {
                x: point.0,
                y: point.1,
            },
        ));
    }
}
