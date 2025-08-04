use macroquad::color::colors;
use nalgebra::{Isometry2, UnitComplex, Vector2};

use crate::shape::Shape;

#[derive(Clone, Copy, Debug)]
pub struct Object {
    pub shape: Shape,

    pub position: Isometry2<f64>,
    pub linear_velocity: Vector2<f64>,
    pub angular_velocity: f64,
}

impl Object {
    pub fn tick(&mut self, dt: f64) {
        self.position
            .append_translation_mut(&(self.linear_velocity * dt).into());
        self.position
            .append_rotation_wrt_center_mut(&UnitComplex::new(self.angular_velocity * dt));
    }

    pub fn draw(&self) {
        self.shape.draw_outline(self.position, 0.1, colors::MAGENTA);
    }

    #[must_use]
    pub fn is_colliding(&self, other: &Self) -> bool {
        self.shape.is_colliding(&other.shape, self.offset_to(other))
    }

    #[must_use]
    pub fn offset_to(&self, other: &Self) -> Isometry2<f64> {
        self.position.inverse() * other.position
    }

    #[must_use]
    pub fn linear_offset_to(&self, other: &Self) -> Vector2<f64> {
        -self.position.translation.vector + other.position.translation.vector
    }
}
