use std::{ops::Deref, sync::LazyLock};

use macroquad::{
    Error,
    camera::Camera2D,
    color::Color,
    input,
    math::Vec2,
    texture::{FilterMode, Image, Texture2D},
    window,
};
use nalgebra::Vector2;

#[must_use]
pub fn vec2_to_f64(vector: Vec2) -> Vector2<f64> {
    <[f32; 2]>::from(vector).map(f64::from).into()
}

#[must_use]
pub fn mouse_position(camera: &Camera2D) -> Vector2<f64> {
    vec2_to_f64(camera.screen_to_world(input::mouse_position().into()))
}

pub fn update_camera_aspect_ratio(camera: &mut Camera2D) {
    camera.zoom.x = camera.zoom.y.abs() * window::screen_height() / window::screen_width();
}

pub fn darken_color(color: Color, brightness: f64) -> Color {
    Color {
        r: color.r * brightness as f32,
        g: color.g * brightness as f32,
        b: color.b * brightness as f32,
        a: color.a,
    }
}

#[derive(Clone, Debug)]
pub struct TextureEntry {
    pub image: Image,
    pub texture: Texture2D,
}

impl TextureEntry {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let image = Image::from_file_with_format(bytes, None)?;
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);

        Ok(Self { image, texture })
    }
}

impl Deref for TextureEntry {
    type Target = Texture2D;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

pub static TEXTURES: LazyLock<[TextureEntry; 5]> = LazyLock::new(|| {
    [
        TextureEntry::from_bytes(include_bytes!("../assets/enemies/red_circle.png")).unwrap(),
        TextureEntry::from_bytes(include_bytes!("../assets/enemies/purple_circle.png")).unwrap(),
        TextureEntry::from_bytes(include_bytes!("../assets/enemies/electric_circle.png")).unwrap(),
        TextureEntry::from_bytes(include_bytes!("../assets/enemies/red_square.png")).unwrap(),
        TextureEntry::from_bytes(include_bytes!("../assets/enemies/purple_square.png")).unwrap(),
    ]
});
