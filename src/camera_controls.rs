use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

#[derive(Component)]
pub struct MainCamera;

pub struct CameraControlsPlugin;

impl Plugin for CameraControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (mouse_drag, mouse_zoom));
    }
}

fn mouse_drag(
    mouse: Res<Input<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    projection_query: Query<&OrthographicProjection, With<MainCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if window_query.single().cursor_position().is_none() {
        return;
    }
    if mouse.pressed(MouseButton::Left) {
        let mut camera_transform = camera_query.single_mut();
        let zoom_level = projection_query.get_single().unwrap().scale;
        for event in motion_evr.iter() {
            camera_transform.translation.x -= event.delta.x * zoom_level;
            camera_transform.translation.y += event.delta.y * zoom_level;
        }
    }
}

fn mouse_zoom(
    mut scroll_evr: EventReader<MouseWheel>,
    mut projection_query: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    let mut projection = projection_query.single_mut();
    for event in scroll_evr.iter() {
        let zoom_delta = match event.y {
            y if y > 0.0 => 1.0 / 1.1,
            y if y < 0.0 => 1.1,
            _ => 1.0,
        };
        projection.scale *= zoom_delta;
    }
}
