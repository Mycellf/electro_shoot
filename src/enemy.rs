use std::ops::{Deref, DerefMut};

use nalgebra::{Isometry2, UnitComplex, vector};

use crate::{object::Object, shape::Shape};

#[derive(Clone, Debug)]
pub struct Enemy {
    pub object: Object,
    pub direction: UnitComplex<f64>,

    pub properties: EnemyProperties,

    pub health: u32,
    pub time_since_hit: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct EnemyProperties {
    pub shape: Shape,

    pub speed: f64,
    pub angular_velocity: f64,

    pub maximum_health: u32,
}

impl Enemy {
    pub const SLOWDOWN_TIME: f64 = 1.0 / 3.0;

    pub fn new(position: Isometry2<f64>, properties: EnemyProperties) -> Self {
        Self {
            object: Object {
                shape: properties.shape,
                position,
                linear_velocity: position.rotation * vector![properties.speed, 0.0],
                angular_velocity: properties.angular_velocity,
            },
            direction: position.rotation,
            properties,
            health: properties.maximum_health,
            time_since_hit: f64::INFINITY,
        }
    }

    pub fn tick(&mut self, dt: f64) {
        let speed = self.properties.speed * (self.time_since_hit / Self::SLOWDOWN_TIME).min(1.0);
        self.object.linear_velocity = self.direction * vector![speed, 0.0];

        self.object.tick(dt);

        self.time_since_hit += dt;
    }

    pub fn hit(&mut self, damage: u32) {
        self.health = self.health.saturating_sub(damage);
        self.time_since_hit = 0.0;
    }

    pub fn should_delete(&self) -> bool {
        self.health == 0
    }
}

impl Deref for Enemy {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl DerefMut for Enemy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}
