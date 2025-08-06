use std::{
    f64::consts::TAU,
    ops::{Deref, DerefMut},
};

use macroquad::{
    color::Color,
    shapes::{self, DrawRectangleParams},
};
use nalgebra::{Isometry2, UnitComplex, Vector2, point, vector};
use slotmap::HopSlotMap;

use crate::{
    enemy::Enemy,
    game::{EnemyKey, ParticleKey},
    object::{Object, Transform},
    particle::Particle,
    shape::Shape,
    utils::{self, GLITTER_TEXTURES},
};

pub static PROJECTILE_KINDS: [ProjectileKind; 3] = [
    ProjectileKind {
        name: "Classic",
        properties: ProjectileProperties {
            size: vector![0.8, 0.2],
            damage: 4,
            piercing: true,
            speed: 15.0,
            particle_distance: 1.0,
            hit_particle_radius: 2,
            hit_particle_distance: 0.8,
        },
        shoot_cooldown: 1.0,
    },
    ProjectileKind {
        name: "Rapid",
        properties: ProjectileProperties {
            size: vector![0.2, 0.2],
            damage: 2,
            piercing: false,
            speed: 30.0,
            particle_distance: 3.0,
            hit_particle_radius: 1,
            hit_particle_distance: 0.8,
        },
        shoot_cooldown: 1.0 / 3.0,
    },
    ProjectileKind {
        name: "Slow",
        properties: ProjectileProperties {
            size: vector![0.4, 0.4],
            damage: 8,
            piercing: true,
            speed: 6.0,
            particle_distance: 0.8,
            hit_particle_radius: 3,
            hit_particle_distance: 0.8 * 2.0 / 3.0,
        },
        shoot_cooldown: 5.0 / 3.0,
    },
];

#[derive(Clone, Debug)]
pub struct Projectile {
    pub object: Object,
    pub direction: UnitComplex<f64>,

    pub properties: ProjectileProperties,

    pub enemies_colliding: Vec<EnemyKey>,
    pub enemies_intersecting: Vec<EnemyKey>,
    pub enemies_hit: Vec<EnemyKey>,
    pub time_since_collision: f64,
    pub time_since_exit: f64,

    pub distance_since_particle: f64,
}

#[derive(Clone, Debug)]
pub struct ProjectileKind {
    pub name: &'static str,
    pub properties: ProjectileProperties,

    pub shoot_cooldown: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct ProjectileProperties {
    pub size: Vector2<f64>,
    pub damage: u32,
    pub piercing: bool,

    pub speed: f64,
    pub particle_distance: f64,

    pub hit_particle_radius: usize,
    pub hit_particle_distance: f64,
}

impl ProjectileProperties {
    pub fn distance_to_front(&self) -> f64 {
        self.size.x / 2.0
    }

    pub fn distance_to_back(&self) -> f64 {
        self.size.x / 2.0
    }
}

impl Projectile {
    pub const COLOR: Color = Color::from_hex(0x00ffff);
    pub const COLLISION_SPEED_MULTIPLIER: f64 = 0.25;
    pub const COLLISION_OPACITY: f64 = 0.75;

    pub const PARTICLE_JITTER: usize = 3;

    pub fn new(position: Isometry2<f64>, kind: &ProjectileKind) -> Self {
        Self {
            object: Object {
                shape: Shape::Rectangle {
                    half_size: kind.properties.size / 2.0,
                },
                transform: Transform {
                    position,
                    linear_velocity: [0.0; 2].into(), // managed each tick
                    angular_velocity: 0.0,
                },
            },
            direction: position.rotation,
            properties: kind.properties,
            enemies_colliding: Vec::new(),
            enemies_intersecting: Vec::new(),
            enemies_hit: Vec::new(),
            time_since_collision: f64::INFINITY,
            time_since_exit: f64::INFINITY,
            distance_since_particle: kind.properties.particle_distance
                - macroquad::rand::gen_range(0, Self::PARTICLE_JITTER) as f64 * 0.1,
        }
    }

    pub fn tick(
        &mut self,
        enemies: &mut HopSlotMap<EnemyKey, Enemy>,
        particles: &mut HopSlotMap<ParticleKey, Particle>,
        dt: f64,
    ) {
        if self.should_delete() {
            return;
        }

        // Motion
        let speed_multiplier = if self.enemies_colliding.is_empty() {
            1.0
        } else {
            Self::COLLISION_SPEED_MULTIPLIER
        };

        let speed = self.properties.speed * speed_multiplier;

        self.object.linear_velocity = self.direction * vector![speed, 0.0];

        self.object.tick(dt);

        // Particles
        self.distance_since_particle += speed * dt;
        while self.distance_since_particle >= self.properties.particle_distance {
            self.distance_since_particle -= self.properties.particle_distance;

            particles.insert(Particle {
                transform: Transform {
                    position: self.position_of_particle(
                        -self.properties.distance_to_back() - self.distance_since_particle + 0.1,
                    ),
                    linear_velocity: vector![0.0, 0.0],
                    angular_velocity: 0.0,
                },
                target_position: None,
                color: Color::from_hex(0x00ffff),
                time_since_creation: 0.0,
                maximum_lifetime: 2.0 / 3.0,
                texture: GLITTER_TEXTURES[macroquad::rand::gen_range(0, GLITTER_TEXTURES.len())]
                    .texture
                    .clone(),
                start: None,
                size: vector![2, 2],
            });
        }

        // Collisions
        self.time_since_collision += dt;

        for (key, enemy) in &mut *enemies {
            if !(self.enemies_intersecting.contains(&key) || self.enemies_colliding.contains(&key))
                && self.object.is_colliding(&enemy.object)
            {
                enemy.hit(self.properties.damage);
                if enemy.should_delete() {
                    enemy.explode(
                        self.position.translation
                            * point![self.properties.distance_to_front(), 0.0],
                        self.linear_velocity / speed_multiplier,
                        particles,
                    );
                } else {
                    self.enemies_colliding.push(key);
                    self.enemies_intersecting.push(key);
                }

                self.add_hit_particles(particles);
                self.enemies_hit.push(key);
                self.time_since_collision = 0.0;
            }
        }

        self.enemies_colliding.retain(|&key| {
            enemies.get(key).is_some_and(|enemy| {
                !enemy.should_delete()
                    && self.object.shape.is_colliding(
                        &enemy.shape,
                        Isometry2::new(
                            -vector![
                                self.properties.distance_to_front()
                                    + self.properties.distance_to_back(),
                                0.0
                            ],
                            0.0,
                        ) * self.object.offset_to(&enemy),
                    )
            })
        });

        self.enemies_intersecting.retain(|&key| {
            enemies
                .get(key)
                .is_some_and(|enemy| !enemy.should_delete() && self.object.is_colliding(&enemy))
        });

        if self.enemies_colliding.is_empty() {
            self.time_since_exit += dt;
        } else {
            self.time_since_exit = 0.0;
        }
    }

    pub fn draw(&self) {
        let opacity = if self.enemies_colliding.is_empty() {
            1.0
        } else {
            Self::COLLISION_OPACITY
        };

        shapes::draw_rectangle_ex(
            self.position.translation.x as f32,
            self.position.translation.y as f32,
            self.properties.size.x as f32,
            self.properties.size.y as f32,
            DrawRectangleParams {
                offset: [0.5, 0.5].into(),
                rotation: self.position.rotation.angle() as f32,
                color: Color {
                    a: opacity as f32,
                    ..utils::brighten_color(Self::COLOR, 1.0 - opacity)
                },
            },
        );
    }

    pub fn add_hit_particles(&self, particles: &mut HopSlotMap<ParticleKey, Particle>) {
        let start_position = self.position_of_particle(self.properties.distance_to_front() - 0.1);

        for target_position in (1..self.properties.hit_particle_radius + 1)
            .map(|x| x as f64 * self.properties.hit_particle_distance)
            .flat_map(|x| [x, -x])
            .map(|x| self.position.rotation * point![0.0, x] + start_position.translation.vector)
        {
            particles.insert(Particle {
                transform: Transform {
                    position: start_position,
                    linear_velocity: vector![0.0, 0.0],
                    angular_velocity: 0.0,
                },
                target_position: Some((target_position, 20.0)),
                color: Color::from_hex(0x00ffff),
                time_since_creation: 0.0,
                maximum_lifetime: 2.0 / 3.0,
                texture: GLITTER_TEXTURES[macroquad::rand::gen_range(0, GLITTER_TEXTURES.len())]
                    .texture
                    .clone(),
                start: None,
                size: vector![2, 2],
            });
        }
    }

    pub fn position_of_particle(&self, offset: f64) -> Isometry2<f64> {
        let translation = self.position * point![offset, 0.0];

        let rotation = self.position.rotation
            * UnitComplex::new(macroquad::rand::gen_range(0, 3) as f64 / 4.0 * TAU);

        Isometry2::from_parts(translation.into(), rotation)
    }

    pub fn should_delete(&self) -> bool {
        !(self.properties.piercing || self.enemies_hit.is_empty())
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
