use std::ops::{Deref, DerefMut};

use macroquad::{
    color::Color,
    shapes::{self, DrawRectangleParams},
};
use nalgebra::{Isometry2, Vector2, point, vector};
use slotmap::HopSlotMap;

use crate::{enemy::Enemy, game::EnemyKey, object::Object, shape::Shape};

#[derive(Clone, Debug)]
pub struct Projectile {
    pub object: Object,

    pub properties: ProjectileProperties,

    pub enemies_intersecting: Vec<EnemyKey>,
    pub time_since_collision: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct ProjectileProperties {
    pub size: Vector2<f64>,
    pub damage: u32,
    pub piercing: bool,

    pub speed: f64,

    pub subticks: usize,
}

impl ProjectileProperties {
    pub fn distance_to_front(&self) -> f64 {
        self.size.y / 2.0
    }
}

impl Projectile {
    pub const COLOR: Color = Color::from_hex(0x00ffff);
    pub const COLLISION_SPEED_MULTIPLIER: f64 = 0.25;
    pub const COLLISION_OPACITY_MULTIPLIER: f64 = 0.75;

    pub fn new(position: Isometry2<f64>, properties: ProjectileProperties) -> Self {
        Self {
            object: Object {
                shape: Shape::Rectangle {
                    half_size: [properties.size.y / 2.0; 2].into(),
                },
                position,
                linear_velocity: [0.0; 2].into(), // managed each tick
                angular_velocity: 0.0,
            },
            properties,
            enemies_intersecting: Vec::new(),
            time_since_collision: f64::INFINITY,
        }
    }

    pub fn tick(&mut self, enemies: &mut HopSlotMap<EnemyKey, Enemy>, dt: f64) {
        let subtick_dt = dt / self.properties.subticks as f64;

        for _ in 0..self.properties.subticks {
            if self.should_delete() {
                return;
            }

            self.subtick(enemies, subtick_dt);
        }
    }

    pub fn draw(&self) {
        let position = self.position * point![self.properties.distance_to_front(), 0.0];

        let opacity = if self.enemies_intersecting.is_empty() {
            1.0
        } else {
            Self::COLLISION_OPACITY_MULTIPLIER
        };

        shapes::draw_rectangle_ex(
            position.x as f32,
            position.y as f32,
            self.properties.size.x as f32,
            self.properties.size.y as f32,
            DrawRectangleParams {
                offset: [1.0, 0.5].into(),
                rotation: self.position.rotation.angle() as f32,
                color: Color {
                    a: opacity as f32,
                    ..Self::COLOR
                },
            },
        );
    }

    pub fn should_delete(&self) -> bool {
        !(self.properties.piercing || self.enemies_intersecting.is_empty())
    }

    pub fn subtick(&mut self, enemies: &mut HopSlotMap<EnemyKey, Enemy>, dt: f64) {
        let speed = if self.enemies_intersecting.is_empty() {
            self.properties.speed
        } else {
            self.properties.speed * Self::COLLISION_SPEED_MULTIPLIER
        };

        self.object.linear_velocity = self.object.position * vector![speed, 0.0];

        self.object.tick(dt);

        for (key, enemy) in &mut *enemies {
            if self.object.is_colliding(&enemy.object) && !self.enemies_intersecting.contains(&key)
            {
                enemy.hit(self.properties.damage);
                self.enemies_intersecting.push(key);
            }
        }

        self.enemies_intersecting.retain(|&key| {
            enemies.get(key).is_some_and(|enemy| {
                !enemy.should_delete() && self.object.is_colliding(&enemy.object)
            })
        });

        if self.enemies_intersecting.is_empty() {
            self.time_since_collision += dt;
        } else {
            self.time_since_collision = 0.0;
        }
    }
}

impl Deref for Projectile {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl DerefMut for Projectile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.object
    }
}
