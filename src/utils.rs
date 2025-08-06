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
use nalgebra::{Point2, Vector2, vector};

use crate::shape::Shape;

#[must_use]
pub fn vec2_to_vector2_f64(vector: Vec2) -> Vector2<f64> {
    <[f32; 2]>::from(vector).map(f64::from).into()
}

#[must_use]
pub fn vec2_to_point2_f64(vector: Vec2) -> Point2<f64> {
    <[f32; 2]>::from(vector).map(f64::from).into()
}

#[must_use]
pub fn vector2_f64_to_vec2(vector: Vector2<f64>) -> Vec2 {
    <[f64; 2]>::from(vector).map(|x| x as f32).into()
}

#[must_use]
pub fn point2_f64_to_vec2(point: Point2<f64>) -> Vec2 {
    <[f64; 2]>::from(point).map(|x| x as f32).into()
}

#[must_use]
pub fn mouse_position(camera: &Camera2D) -> Point2<f64> {
    vec2_to_point2_f64(camera.screen_to_world(input::mouse_position().into()))
}

pub fn update_camera_aspect_ratio(camera: &mut Camera2D) {
    camera.zoom.x = camera.zoom.y.abs() * window::screen_height() / window::screen_width();
}

#[must_use]
pub fn bounds_of_camera(camera: &Camera2D) -> Shape {
    Shape::Rectangle {
        half_size: vec2_to_vector2_f64(1.0 / camera.zoom).abs(),
    }
}

pub fn darken_color(color: Color, brightness: f64) -> Color {
    Color {
        r: color.r * brightness as f32,
        g: color.g * brightness as f32,
        b: color.b * brightness as f32,
        a: color.a,
    }
}

pub fn brighten_color(color: Color, brightness: f64) -> Color {
    Color {
        r: color.r + brightness as f32,
        g: color.g + brightness as f32,
        b: color.b + brightness as f32,
        a: color.a,
    }
}

pub fn next_flickering_brightness(current_brightnes: f64, minimum_brightness: f64) -> f64 {
    if minimum_brightness == 1.0 {
        1.0
    } else if minimum_brightness > 0.5 {
        macroquad::rand::gen_range(minimum_brightness, (minimum_brightness + 0.75).min(1.0))
    } else if (current_brightnes < 0.5) ^ (macroquad::rand::rand() & 0b11 == 0) {
        macroquad::rand::gen_range(0.5, (minimum_brightness + 0.75).min(1.0))
    } else {
        macroquad::rand::gen_range(minimum_brightness, 0.5)
    }
}

#[must_use]
pub const fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[must_use]
pub const fn color_lerp(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: lerp(a.r, b.r, t),
        g: lerp(a.g, b.g, t),
        b: lerp(a.b, b.b, t),
        a: lerp(a.a, b.a, t),
    }
}

/// CREDIT: Freya Holmér: <https://www.youtube.com/watch?v=LSNQuFEDOyQ>
#[must_use]
pub fn exp_decay(a: f64, b: f64, decay: f64, dt: f64) -> f64 {
    b + (a - b) * (-decay * dt).exp()
}

/// CREDIT: Freya Holmér: <https://www.youtube.com/watch?v=LSNQuFEDOyQ>
#[must_use]
pub fn lerp_follow(a: f64, b: f64, t: f64, dt: f64) -> f64 {
    b + (a - b) * t.powf(dt)
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

    pub fn pixel_size(&self) -> Vector2<usize> {
        vector![self.image.width(), self.image.height()]
    }
}

impl Deref for TextureEntry {
    type Target = Texture2D;

    fn deref(&self) -> &Self::Target {
        &self.texture
    }
}

pub static TURRET_BASE_TEXTURE: LazyLock<TextureEntry> = LazyLock::new(|| {
    TextureEntry::from_bytes(include_bytes!("../assets/turret/base.png")).unwrap()
});

pub static ENEMY_TEXTURES: LazyLock<[TextureEntry; 5]> = LazyLock::new(|| {
    [
        TextureEntry::from_bytes(include_bytes!("../assets/enemies/red_circle.png")).unwrap(),
        TextureEntry::from_bytes(include_bytes!("../assets/enemies/purple_circle.png")).unwrap(),
        TextureEntry::from_bytes(include_bytes!("../assets/enemies/electric_circle.png")).unwrap(),
        TextureEntry::from_bytes(include_bytes!("../assets/enemies/red_square.png")).unwrap(),
        TextureEntry::from_bytes(include_bytes!("../assets/enemies/purple_square.png")).unwrap(),
    ]
});

pub static GLITTER_TEXTURES: LazyLock<[TextureEntry; 2]> = LazyLock::new(|| {
    [
        TextureEntry::from_bytes(include_bytes!("../assets/particles/glitter_1.png")).unwrap(),
        TextureEntry::from_bytes(include_bytes!("../assets/particles/glitter_2.png")).unwrap(),
    ]
});

pub static ABSORB_TEXTURE: LazyLock<TextureEntry> = LazyLock::new(|| {
    TextureEntry::from_bytes(include_bytes!("../assets/particles/absorb.png")).unwrap()
});

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pub min: Point2<usize>,
    pub max: Point2<usize>,
}

impl BoundingBox {
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.x <= other.max.x
            && self.min.y <= other.max.y
            && other.min.x <= self.max.x
            && other.min.y <= self.max.y
    }

    pub fn center(&self) -> Point2<f64> {
        (self.min.map(|x| x as f64) + self.max.map(|x| (x + 1) as f64).coords) / 2.0
    }

    pub fn size(&self) -> Vector2<usize> {
        self.max - self.min + vector![1, 1]
    }

    pub fn combine(self, other: BoundingBox) -> BoundingBox {
        BoundingBox {
            min: Vector2::from_fn(|i, _| self.min[i].min(other.min[i])).into(),
            max: Vector2::from_fn(|i, _| self.max[i].max(other.max[i])).into(),
        }
    }
}
