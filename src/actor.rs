use bevy::{prelude::*, utils::HashMap};
use rand::{thread_rng, Rng};

use crate::{
    level_generation::map::Map,
    sprite_atlas::{SpriteAtlas, SpriteIndex}, world_to_map_index,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
enum TurnState {
 #[default]
  Player,
  Enemy
}
pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<TurnState>()
            .add_systems(PostStartup, (spawn_player, spawn_enemies))
            .add_systems(Update, (player_movement.run_if(state_exists_and_equals(TurnState::Player)), center_camera_on_player).chain())
            .add_systems(Update, enemy_movement.run_if(state_exists_and_equals(TurnState::Enemy)));
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
    mut player_query: Query<(&mut Movement, &mut Transform), (With<Player>, Without<Enemy>)>,
    enemy_query: Query<&Transform, (With<Enemy>, Without<Player>)>,
    map: Res<Map>,
    keyboard: Res<Input<KeyCode>>,
    mut next_state: ResMut<NextState<TurnState>>
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
        let player_index = world_to_map_index(&player_transform);
        let new_index = (player_index.0.saturating_add_signed(delta[0]), player_index.1.saturating_add_signed(delta[1]));


        //check that player won't collide with any enemies
        if enemy_query.iter().any(|transform| {
            world_to_map_index(transform) == new_index
        }) {
            return;
        }

        if let Some(tile) = map.get(new_index.0, new_index.1) {
            if tile.passable {
                player_transform.translation.x += delta[0] as f32 * 12.0;
                player_transform.translation.y += delta[1] as f32 * 12.0;
                eprintln!("Player Moved: ({:?}, {:?})", delta[0], delta[1]);
                eprintln!("Player Translation: {:?}", player_transform.translation);
                player_movement.just_moved = true;
                next_state.set(TurnState::Enemy);
            }
        }
    }
}


fn enemy_movement(
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<(Entity, &mut Transform), (With<Enemy>, Without<Player>)>,
    map: Res<Map>,
    mut next_state: ResMut<NextState<TurnState>>
) {
    let mut rng = thread_rng();
    
    let mut deltas = HashMap::new();
    let mut collides = HashMap::new();
    enemy_query.iter().for_each(|(entity, _)| {
        deltas.insert(entity, (rng.gen_range(-1..=1), rng.gen_range(-1..=1)));
        collides.insert(entity, false);
    });

    //check for any collisions between enemies
    for [(entity1, trans1), (entity2, trans2)] in enemy_query.iter_combinations() {
        let old_index1 = world_to_map_index(&trans1);
        let delta1 = deltas.get(&entity1).unwrap();
        let new_index1 = (old_index1.0.saturating_add_signed(delta1.0), old_index1.1.saturating_add_signed(delta1.1));

        let old_index2 = world_to_map_index(&trans2);
        let delta2 = deltas.get(&entity2).unwrap();
        let new_index2 = (old_index2.0.saturating_add_signed(delta2.0), old_index2.1.saturating_add_signed(delta2.1));

        if new_index1 == new_index2 || old_index1 == new_index2 || old_index2 == new_index1{
            collides.insert(entity1, true);
            collides.insert(entity2, true);
        }
    }

    //player collisions
    for (entity, transform) in enemy_query.iter() {
        let old_index = world_to_map_index(&transform);
        let delta = deltas.get(&entity).unwrap();
        let new_index = (old_index.0.saturating_add_signed(delta.0), old_index.1.saturating_add_signed(delta.1));
        for player_transform in player_query.iter() {
            let player_index = world_to_map_index(&player_transform);
            if new_index == player_index {
                collides.insert(entity, true);
            }
        }
    }

    for (entity, mut transform) in enemy_query.iter_mut() {
        println!("Collision Detected?");
        if !collides.get(&entity).unwrap() {
            println!("Nope!");
            let old_index = world_to_map_index(&transform);
            let delta = deltas.get(&entity).unwrap();
            let new_index = (old_index.0.saturating_add_signed(delta.0), old_index.1.saturating_add_signed(delta.1));
            if let Some(tile) = map.get(new_index.0, new_index.1) {
                if tile.passable {
                    transform.translation.x += delta.0 as f32 * 12.0;
                    transform.translation.y += delta.1 as f32 * 12.0;
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
