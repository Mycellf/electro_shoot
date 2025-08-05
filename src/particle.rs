use std::ops::{Deref, DerefMut};

use macroquad::{
    color::Color,
    math::Rect,
    texture::{self, DrawTextureParams, Texture2D},
};
use nalgebra::{Point2, Vector2};

use crate::{object::Transform, utils};

#[derive(Clone, Debug)]
pub struct Particle {
    pub transform: Transform,
    pub target_position: Option<(Point2<f64>, f64)>,

    pub color: Color,
    pub time_since_creation: f64,
    pub maximum_lifetime: f64,

    pub texture: Texture2D,

    pub start: Option<Point2<usize>>,
    pub size: Vector2<usize>,
}

impl Particle {
    pub fn tick(&mut self, dt: f64) {
        self.time_since_creation += dt;

        self.transform.tick(dt);

        if let Some((target_position, decay_speed)) = self.target_position {
            self.transform.position.translation.vector =
                self.transform.position.translation.vector.lerp(
                    &target_position.coords,
                    utils::exp_decay(0.0, 1.0, decay_speed, dt),
                );
        }
    }

    pub fn draw(&self) {
        let size = self.size.map(|x| x as f64) * 0.1;

        texture::draw_texture_ex(
            &self.texture,
            (self.position.translation.x - size.x / 2.0) as f32,
            (self.position.translation.y - size.y / 2.0) as f32,
            Color {
                a: (1.0 - self.time_since_creation / self.maximum_lifetime) as f32,
                ..self.color
            },
            DrawTextureParams {
                dest_size: Some(utils::vector2_f64_to_vec2(size)),
                source: self.start.map(|start| Rect {
                    x: start.x as f32,
                    y: start.y as f32,
                    w: self.size.x as f32,
                    h: self.size.y as f32,
                }),
                rotation: self.position.rotation.angle() as f32,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );
    }

    pub fn should_delete(&self) -> bool {
        self.time_since_creation >= self.maximum_lifetime
    }
}

impl Deref for Particle {
    type Target = Transform;

    fn deref(&self) -> &Self::Target {
        &self.transform
    }
}

impl DerefMut for Particle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transform
    }
}
