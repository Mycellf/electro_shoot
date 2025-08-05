use macroquad::{
    color::{Color, colors},
    input::{self, KeyCode, MouseButton},
    shapes::{self, DrawRectangleParams},
    texture::{self, DrawTextureParams},
};
use nalgebra::{Complex, Isometry2, Point2, UnitComplex, point, vector};
use slotmap::HopSlotMap;

use crate::{
    game::ProjectileKey,
    projectile::{PROJECTILE_KINDS, Projectile, ProjectileKind},
    shape::Shape,
    utils::{self, TURRET_BASE_TEXTURE},
};

#[derive(Clone, Debug)]
pub struct Turret {
    pub shape: Shape,
    pub position: Isometry2<f64>,

    pub time_since_shoot: f64,
    pub time_since_recharged: f64,
    pub projectile_kind: ProjectileKind,

    pub input: PlayerInput,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerInput {
    pub shoot: bool,
    pub time_since_press: f64,
}

impl Default for Turret {
    fn default() -> Self {
        Self {
            shape: Shape::Circle { radius: 0.6 },
            position: Isometry2::new(vector![0.0, 0.0], 0.0),
            time_since_shoot: 0.0,
            time_since_recharged: 0.0,
            projectile_kind: PROJECTILE_KINDS[0].clone(),
            input: PlayerInput::default(),
        }
    }
}

impl PlayerInput {
    pub const KEYS: [KeyCode; 1] = [KeyCode::Space];
    pub const MOUSE_BUTTONS: [MouseButton; 2] = [MouseButton::Left, MouseButton::Right];

    pub const BUFFER_TIME: f64 = 1.0 / 6.0;

    pub fn tick(&mut self, dt: f64) {
        let _ = dt;

        if Self::shoot_down() {
            self.shoot = true;
            self.time_since_press = 0.0;
        } else {
            self.time_since_press += dt;

            if self.time_since_press > Self::BUFFER_TIME {
                self.shoot = false;
            }
        }
    }

    pub fn shoot_down() -> bool {
        Self::KEYS.into_iter().any(input::is_key_down)
            || Self::MOUSE_BUTTONS.into_iter().any(|button| {
                input::is_mouse_button_down(button) || input::is_mouse_button_pressed(button)
            })
    }
}

impl Turret {
    pub const PLATFORM_WIDTH: f64 = 0.4;
    pub const PLATFORM_RECHARGE_ANIMATION_WIDTH: f64 = 0.6;
    pub const RECHARGE_ANIMATION_LENGTH: f64 = 0.5;

    pub const BARREL_LENGTH: f64 = 1.0;
    pub const BARREL_WIDTH: f64 = 0.2;

    pub const BARREL_BASE_OFFSET: f64 = Self::BARREL_WIDTH / 2.0;
    pub const BARREL_SHOOT_OFFSET: f64 = 0.5;

    pub fn tick(
        &mut self,
        mouse_position: Point2<f64>,
        projectiles: &mut HopSlotMap<ProjectileKey, Projectile>,
        dt: f64,
    ) {
        let mouse_offset = mouse_position.coords - self.position.translation.vector;

        let mouse_direction = if mouse_offset.magnitude_squared() == 0.0 {
            self.position.rotation
        } else {
            UnitComplex::new_normalize(Complex::new(mouse_offset.x, mouse_offset.y))
        };

        self.time_since_shoot += dt;

        if self.input.shoot && self.can_shoot() {
            self.position.rotation = mouse_direction;
            self.shoot(projectiles);
        } else {
            self.position.rotation = (self.position.rotation)
                .slerp(&mouse_direction, utils::exp_decay(0.0, 1.0, 20.0, dt));
        }

        if self.can_shoot() {
            self.time_since_recharged += dt;
        } else {
            self.time_since_recharged = 0.0;
        }
    }

    pub fn draw(&self) {
        let size = TURRET_BASE_TEXTURE.size() * 0.1;

        texture::draw_texture_ex(
            &TURRET_BASE_TEXTURE,
            self.position.translation.x as f32 - size.x / 2.0,
            self.position.translation.y as f32 - size.y / 2.0,
            colors::WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                source: None,
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );

        let show_animation = self.show_recharge_animation();

        let brightness = if show_animation { 0.5 } else { 0.0 };

        let width = if show_animation {
            Self::PLATFORM_RECHARGE_ANIMATION_WIDTH
        } else {
            Self::PLATFORM_WIDTH
        };

        shapes::draw_rectangle_ex(
            self.position.translation.x as f32,
            self.position.translation.y as f32,
            width as f32,
            width as f32,
            DrawRectangleParams {
                offset: [0.5, 0.5].into(),
                rotation: self.position.rotation.angle() as f32,
                color: utils::brighten_color(Color::from_hex(0x00b6bf), brightness),
            },
        );

        let position = self.position
            * point![
                -Self::BARREL_BASE_OFFSET - self.shoot_recharge_offset(),
                0.0
            ];

        shapes::draw_rectangle_ex(
            position.x as f32,
            position.y as f32,
            (Self::BARREL_LENGTH + Self::BARREL_BASE_OFFSET) as f32,
            Self::BARREL_WIDTH as f32,
            DrawRectangleParams {
                offset: [0.0, 0.5].into(),
                rotation: self.position.rotation.angle() as f32,
                color: utils::brighten_color(
                    Color::from_hex(0x00d8e4),
                    (1.0 - self.shoot_recharge_progress()) * 0.65,
                ),
            },
        )
    }

    pub fn shoot(&mut self, projectiles: &mut HopSlotMap<ProjectileKey, Projectile>) {
        self.time_since_shoot = 0.0;
        self.input.shoot = false;

        let translation = self.position
            * point![
                Self::BARREL_LENGTH + self.projectile_kind.properties.distance_to_front(),
                0.0
            ];
        let position = Isometry2::from_parts(translation.into(), self.position.rotation);

        projectiles.insert(Projectile::new(position, &self.projectile_kind));
    }

    pub fn shoot_recharge_progress(&self) -> f64 {
        (self.time_since_shoot / self.projectile_kind.shoot_cooldown).clamp(0.0, 1.0)
    }

    pub fn shoot_recharge_offset(&self) -> f64 {
        let progress = (1.0 - self.shoot_recharge_progress()).powi(2);
        let offset = progress * Self::BARREL_SHOOT_OFFSET;

        (offset / 0.1).ceil() * 0.1
    }

    pub fn can_shoot(&self) -> bool {
        self.time_since_shoot >= self.projectile_kind.shoot_cooldown
    }

    pub fn show_recharge_animation(&self) -> bool {
        self.can_shoot() && self.time_since_recharged < Self::RECHARGE_ANIMATION_LENGTH
    }
}
