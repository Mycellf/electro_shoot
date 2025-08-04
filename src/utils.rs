use macroquad::{camera::Camera2D, input, window};
use nalgebra::Vector2;

pub fn mouse_position(camera: &Camera2D) -> Vector2<f64> {
    <[f32; 2]>::from(camera.screen_to_world(input::mouse_position().into()))
        .map(|x| x as f64)
        .into()
}

pub fn update_camera_aspect_ratio(camera: &mut Camera2D) {
    camera.zoom.x = camera.zoom.y.abs() * window::screen_height() / window::screen_width();
}
