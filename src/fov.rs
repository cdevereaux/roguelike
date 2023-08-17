use crate::{
    actor::Player,
    level_generation::{
        map::{Map, ViewStatus},
        MapTile,
    },
};
use bevy::prelude::*;

pub struct FovPlugin;

impl Plugin for FovPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (reset_fov, calculate_fov, update_fov).chain());
    }
}

fn reset_fov(mut map: ResMut<Map>) {
    for x in 0..map.width {
        for y in 0..map.height {
            let tile = map.get_mut(x, y).unwrap();
            tile.view_status = match tile.view_status {
                ViewStatus::Revealed => ViewStatus::Revealed,
                ViewStatus::Seen => ViewStatus::Revealed,
                ViewStatus::Unexplored => ViewStatus::Unexplored,
            }
        }
    }
}

fn update_fov(
    map: Res<Map>,
    mut query: Query<(&mut TextureAtlasSprite, &Transform), With<MapTile>>,
) {
    for (mut sprite, transform) in query.iter_mut() {
        let x = transform.translation.x as usize / 12;
        let y = transform.translation.y as usize / 12;
        sprite.color = match map.get(x, y).unwrap().view_status {
            ViewStatus::Seen => Color::WHITE,
            ViewStatus::Revealed => Color::GRAY,
            ViewStatus::Unexplored => Color::BLACK,
        };
    }
}

//Port of shadow-casting code from here: https://www.roguebasin.com/index.php/Python_shadowcasting_implementation
fn calculate_fov(mut map: ResMut<Map>, query: Query<&Transform, With<Player>>) {
    const MULTIPLIERS: [[isize; 8]; 4] = [
        [1, 0, 0, -1, -1, 0, 0, 1],
        [0, 1, -1, 0, 0, -1, 1, 0],
        [0, 1, 1, 0, 0, -1, -1, 0],
        [1, 0, 0, 1, -1, 0, 0, -1],
    ];

    let radius = 15;
    let player_transform = query.single();
    let x0 = player_transform.translation.x as isize / 12;
    let y0 = player_transform.translation.y as isize / 12;

    for octant in 0..8 {
        cast_light(
            &mut map,
            x0,
            y0,
            1,
            1.0,
            0.0,
            radius,
            MULTIPLIERS[0][octant],
            MULTIPLIERS[1][octant],
            MULTIPLIERS[2][octant],
            MULTIPLIERS[3][octant],
            0,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn cast_light(
    map: &mut Map,
    x0: isize,
    y0: isize,
    row: isize,
    mut start: f32,
    end: f32,
    radius: isize,
    xx: isize,
    xy: isize,
    yx: isize,
    yy: isize,
    _depth: usize,
) {
    if start < end {
        return;
    }

    let radius_squared = radius * radius;
    for j in row..=radius {
        let (mut dx, dy) = (-j - 1, -j);
        let mut blocked = false;
        let mut new_start = 0.0;
        while dx <= 0 {
            dx += 1;
            //Translate the dx, dy coordinates into map coordinates:
            let (x, y) = (x0 + dx * xx + dy * xy, y0 + dx * yx + dy * yy);
            // l_slope and r_slope store the slopes of the left and right
            // extremities of the square we're considering:
            let (l_slope, r_slope) = (
                (dx as f32 - 0.5) / (dy as f32 + 0.5),
                (dx as f32 + 0.5) / (dy as f32 - 0.5),
            );
            if start < r_slope {
                continue;
            } else if end > l_slope {
                break;
            } else {
                // Our light beam is touching this square; light it:
                if dx * dx + dy * dy < radius_squared && x > 0 && y > 0 {
                    if let Some(tile) = map.get_mut(x as usize, y as usize) {
                        tile.view_status = ViewStatus::Seen;
                    }
                }
                if blocked {
                    // we're scanning a row of blocked squares:
                    if is_blocked(map, x, y) {
                        new_start = r_slope;
                        continue;
                    } else {
                        blocked = false;
                        start = new_start;
                    }
                } else if is_blocked(map, x, y) && j < radius {
                    // This is a blocking square, start a child scan:
                    blocked = true;
                    cast_light(
                        map,
                        x0,
                        y0,
                        j + 1,
                        start,
                        l_slope,
                        radius,
                        xx,
                        xy,
                        yx,
                        yy,
                        _depth + 1,
                    );
                    new_start = r_slope
                }
            }
        }
        if blocked {
            break;
        }
    }
}

fn is_blocked(map: &Map, x: isize, y: isize) -> bool {
    return x < 0
        || y < 0
        || if let Some(tile) = map.get(x as usize, y as usize) {
            !tile.passable
        } else {
            true
        };
}
