use std::{
    f64::consts::TAU,
    ops::{Deref, DerefMut},
    sync::LazyLock,
};

use macroquad::{
    color::colors,
    texture::{self, DrawTextureParams},
};
use nalgebra::{Isometry2, UnitComplex, vector};

use crate::{
    object::{Object, Transform},
    shape::Shape,
    utils::{self, ENEMY_TEXTURES, TextureEntry},
};

pub static ENEMY_KINDS: LazyLock<[EnemyKind; 5]> = LazyLock::new(|| {
    [
        EnemyKind {
            name: "Red Circle",
            properties: EnemyProperties {
                shape: Shape::Circle { radius: 0.5 },
                speed: 3.0,
                angular_velocity: 0.0,
                maximum_health: 4,
                texture: &ENEMY_TEXTURES[0],
            },
        },
        EnemyKind {
            name: "Purple Circle",
            properties: EnemyProperties {
                shape: Shape::Circle { radius: 0.5 },
                speed: 9.0,
                angular_velocity: 0.0,
                maximum_health: 4,
                texture: &ENEMY_TEXTURES[1],
            },
        },
        EnemyKind {
            name: "Electric Circle",
            properties: EnemyProperties {
                shape: Shape::Circle { radius: 0.6 },
                speed: 12.0,
                angular_velocity: 0.0,
                maximum_health: 4,
                texture: &ENEMY_TEXTURES[2],
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
                texture: &ENEMY_TEXTURES[3],
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
                texture: &ENEMY_TEXTURES[4],
            },
        },
    ]
});

#[derive(Clone, Debug)]
pub struct Enemy {
    pub object: Object,
    pub direction: UnitComplex<f64>,

    pub properties: EnemyProperties,

    pub health: u32,
    pub time_since_hit: f64,

    pub brightness: f64,
    pub brightness_update_time: f64,
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
    pub texture: &'static TextureEntry,
}

impl Enemy {
    pub const SLOWDOWN_TIME: f64 = 1.0 / 3.0;

    pub fn new(position: Isometry2<f64>, kind: &EnemyKind) -> Self {
        Self {
            object: Object {
                shape: kind.properties.shape,
                transform: Transform {
                    position,
                    linear_velocity: vector![0.0, 0.0], // managed each tick
                    angular_velocity: kind.properties.angular_velocity,
                },
            },
            direction: position.rotation,
            properties: kind.properties,
            health: kind.properties.maximum_health,
            time_since_hit: f64::INFINITY,
            brightness: 0.0,
            brightness_update_time: 0.0,
        }
    }

    pub fn tick(&mut self, dt: f64) {
        let speed = self.properties.speed * self.speed_multiplier();
        self.object.linear_velocity = self.direction * vector![speed, 0.0];

        self.object.tick(dt);

        self.brightness_update_time += dt * 30.0;

        if self.brightness_update_time > 1.0 {
            self.brightness_update_time %= 1.0;
            let brightness = self.speed_multiplier();
            self.brightness = macroquad::rand::gen_range(brightness, (brightness + 0.75).min(1.0));
        }

        self.time_since_hit += dt;
    }

    pub fn draw(&self) {
        let size = self.properties.texture.size() * 0.1;

        texture::draw_texture_ex(
            &self.properties.texture,
            self.position.translation.x as f32 - size.x / 2.0,
            self.position.translation.y as f32 - size.y / 2.0,
            utils::darken_color(colors::WHITE, self.brightness),
            DrawTextureParams {
                dest_size: Some(size),
                source: None,
                rotation: self.position.rotation.angle() as f32,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );
    }

    pub fn speed_multiplier(&self) -> f64 {
        (self.time_since_hit / Self::SLOWDOWN_TIME).min(1.0)
    }

    pub fn hit(&mut self, damage: u32) {
        self.health = self.health.saturating_sub(damage);
        self.time_since_hit = 0.0;
        self.brightness_update_time = 1.0;
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
