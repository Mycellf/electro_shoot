use std::{
    f64::consts::TAU,
    ops::{Deref, DerefMut},
    sync::LazyLock,
};

use macroquad::{
    color::colors,
    texture::{self, DrawTextureParams, FilterMode, Image, Texture2D},
};
use nalgebra::{DMatrix, Isometry2, Point2, UnitComplex, Vector2, point, vector};
use slotmap::{HopSlotMap, SlotMap};

use crate::{
    game::ParticleKey,
    object::{Object, Transform},
    particle::Particle,
    shape::Shape,
    utils::{self, BoundingBox, ENEMY_TEXTURES, TextureEntry},
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
            self.brightness =
                utils::next_flickering_brightness(self.brightness, self.speed_multiplier());
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

    pub fn explode(
        &self,
        hit_position: Point2<f64>,
        hit_velocity: Vector2<f64>,
        particles: &mut HopSlotMap<ParticleKey, Particle>,
    ) {
        let size = self.properties.texture.pixel_size();

        let mut num_valid_pixels = (self.properties.texture.image)
            .get_image_data()
            .iter()
            .filter(|&&[_, _, _, opacity]| opacity > 0)
            .count();

        let mut groups = DMatrix::from_element(size.x, size.y, None);
        let mut bounding_boxes = SlotMap::new();

        while num_valid_pixels > 0 {
            let mut count = macroquad::rand::gen_range(1, num_valid_pixels);

            let index = groups
                .iter()
                .zip(self.properties.texture.image.get_image_data())
                .take_while(|(group, [_, _, _, opacity])| {
                    if group.is_none() && *opacity > 0 {
                        count -= 1;
                    }

                    count > 0
                })
                .count();

            let position = point![index % size.x, index / size.x];

            bounding_boxes.insert_with_key(|group| {
                let mut full_bounding_box = None;

                for _ in 0..macroquad::rand::gen_range(1, 3) {
                    let rectangle_size = vector![
                        macroquad::rand::gen_range(2, 5),
                        macroquad::rand::gen_range(2, 5)
                    ];

                    let rectangle_offset = vector![
                        macroquad::rand::gen_range(0, rectangle_size.x),
                        macroquad::rand::gen_range(0, rectangle_size.y),
                    ];

                    let bounding_box = BoundingBox {
                        min: Vector2::from_fn(|i, _| {
                            position[i].saturating_sub(rectangle_offset[i])
                        })
                        .into(),
                        max: Vector2::from_fn(|i, _| {
                            (position[i] + rectangle_size[i]).min(size[i]) - 1
                        })
                        .into(),
                    };

                    for x in bounding_box.min.x..bounding_box.max.x + 1 {
                        for y in bounding_box.min.y..bounding_box.max.y + 1 {
                            if groups[(x, y)].is_none()
                                && (self.properties.texture.image)
                                    .get_pixel(x as u32, y as u32)
                                    .a
                                    > f32::EPSILON
                            {
                                groups[(x, y)] = Some(group);
                                num_valid_pixels -= 1;
                            }
                        }
                    }

                    full_bounding_box = full_bounding_box
                        .map_or(Some(bounding_box), |full| Some(bounding_box.combine(full)));
                }

                full_bounding_box.unwrap()
            });
        }

        while let Some(key) = bounding_boxes.keys().next() {
            let mut texture_bounding_boxes = vec![(key, bounding_boxes.remove(key).unwrap())];

            bounding_boxes.retain(|group, bounding_box| {
                if texture_bounding_boxes
                    .iter()
                    .any(|(_, other)| bounding_box.intersects(other))
                {
                    true
                } else {
                    texture_bounding_boxes.push((group, *bounding_box));
                    false
                }
            });

            let mut image = Image::gen_image_color(size.x as u16, size.y as u16, colors::BLANK);

            for &(group, bounding_box) in &texture_bounding_boxes {
                for x in bounding_box.min.x..bounding_box.max.x + 1 {
                    for y in bounding_box.min.y..bounding_box.max.y + 1 {
                        let i = x + y * size.x;

                        if groups[(x, y)] == Some(group) {
                            image.get_image_data_mut()[i] =
                                self.properties.texture.image.get_image_data()[i];
                        }
                    }
                }
            }

            let texture = Texture2D::from_image(&image);
            texture.set_filter(FilterMode::Nearest);

            for (_, bounding_box) in texture_bounding_boxes {
                let offset = 0.1 * (bounding_box.center() - size.map(|x| x as f64) / 2.0);

                let translation = self.position * offset;

                let additional_velocity = (translation - hit_position).normalize();

                particles.insert(Particle {
                    transform: Transform {
                        position: Isometry2::from_parts(translation.into(), self.position.rotation),
                        linear_velocity: self.velocity_of_point(translation) - self.linear_velocity
                            + (additional_velocity + hit_velocity * 0.1)
                                * macroquad::rand::gen_range(0.1, 1.0),
                        angular_velocity: self.angular_velocity,
                    },
                    target_position: None,
                    color: colors::WHITE,
                    time_since_creation: 0.0,
                    maximum_lifetime: 1.0,
                    texture: texture.clone(),
                    start: Some(bounding_box.min),
                    size: bounding_box.size(),
                });
            }
        }
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
