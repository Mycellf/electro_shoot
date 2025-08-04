use macroquad::{camera::Camera2D, input, math::Vec2, window};
use nalgebra::Vector2;

#[must_use] pub fn vec2_to_f64(vector: Vec2) -> Vector2<f64> {
    <[f32; 2]>::from(vector).map(f64::from).into()
}

#[must_use] pub fn mouse_position(camera: &Camera2D) -> Vector2<f64> {
    vec2_to_f64(camera.screen_to_world(input::mouse_position().into()))
}

pub fn update_camera_aspect_ratio(camera: &mut Camera2D) {
    camera.zoom.x = camera.zoom.y.abs() * window::screen_height() / window::screen_width();
}
