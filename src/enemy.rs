use std::{
    f64::consts::TAU,
    ops::{Deref, DerefMut},
};

use nalgebra::{Isometry2, UnitComplex, vector};

use crate::{object::Object, shape::Shape};

pub static ENEMY_KINDS: [EnemyKind; 5] = [
    EnemyKind {
        name: "Red Circle",
        properties: EnemyProperties {
            shape: Shape::Circle { radius: 0.5 },
            speed: 3.0,
            angular_velocity: 0.0,
            maximum_health: 4,
        },
    },
    EnemyKind {
        name: "Purple Circle",
        properties: EnemyProperties {
            shape: Shape::Circle { radius: 0.5 },
            speed: 9.0,
            angular_velocity: 0.0,
            maximum_health: 4,
        },
    },
    EnemyKind {
        name: "Electric Circle",
        properties: EnemyProperties {
            shape: Shape::Circle { radius: 0.6 },
            speed: 12.0,
            angular_velocity: 0.0,
            maximum_health: 4,
        },
    },
    EnemyKind {
        name: "Red Square",
        properties: EnemyProperties {
            shape: Shape::Rectangle {
                half_size: vector![0.6, 0.6],
            },
            speed: 3.0,
            angular_velocity: -5.0 / 24.0 * TAU,
            maximum_health: 8,
        },
    },
    EnemyKind {
        name: "Purple Square",
        properties: EnemyProperties {
            shape: Shape::Rectangle {
                half_size: vector![0.8, 0.8],
            },
            speed: 3.0,
            angular_velocity: 1.0 / 6.0 * TAU,
            maximum_health: 12,
        },
    },
];

#[derive(Clone, Debug)]
pub struct Enemy {
    pub object: Object,
    pub direction: UnitComplex<f64>,

    pub properties: EnemyProperties,

    pub health: u32,
    pub time_since_hit: f64,
}

#[derive(Clone, Debug)]
pub struct EnemyKind {
    pub name: &'static str,
    pub properties: EnemyProperties,
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

    pub fn new(position: Isometry2<f64>, kind: &EnemyKind) -> Self {
        Self {
            object: Object {
                shape: kind.properties.shape,
                position,
                linear_velocity: position.rotation * vector![kind.properties.speed, 0.0],
                angular_velocity: kind.properties.angular_velocity,
            },
            direction: position.rotation,
            properties: kind.properties,
            health: kind.properties.maximum_health,
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
