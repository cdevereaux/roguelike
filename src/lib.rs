use bevy::prelude::*;

pub mod actor;
pub mod camera_controls;
pub mod fov;
pub mod level_generation;
pub mod position;
pub mod sprite_atlas;

pub fn world_to_map_index(transform: &Transform) -> (usize, usize) {
    (
        transform.translation.x as usize / 12,
        transform.translation.y as usize / 12,
    )
}
