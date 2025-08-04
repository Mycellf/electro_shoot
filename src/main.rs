pub mod enemy;
pub mod game;
pub mod object;
pub mod projectile;
pub mod shape;
pub mod utils;

use std::f64::consts::TAU;

use macroquad::{
    camera::{self, Camera2D},
    input::{self, KeyCode},
    window::{self, Conf},
};
use nalgebra::{Isometry2, vector};

use crate::{
    enemy::Enemy,
    game::Game,
    object::Object,
    projectile::{Projectile, ProjectileProperties},
    shape::Shape,
};

const START_IN_FULLSCREEN: bool = false;

fn config() -> Conf {
    Conf {
        window_title: "Electro Shoot".to_owned(),
        fullscreen: START_IN_FULLSCREEN,
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    let mut fullscreen = START_IN_FULLSCREEN;

    let screen_height = 36.0;
    let mut camera = Camera2D {
        zoom: [-2.0 / screen_height as f32; 2].into(),
        ..Default::default()
    };

    let mut game = Game::default();

    game.projectiles.insert(Projectile::new(
        Isometry2::new(vector![0.0, 0.0], 0.0),
        ProjectileProperties {
            size: vector![0.8, 0.2],
            damage: 4,
            piercing: true,
            speed: 5.0,
            subticks: 2,
        },
    ));

    game.projectiles.insert(Projectile::new(
        Isometry2::new(vector![-10.0, 0.0], 0.0),
        ProjectileProperties {
            size: vector![0.8, 0.2],
            damage: 4,
            piercing: true,
            speed: 5.0,
            subticks: 2,
        },
    ));

    game.enemies.insert(Enemy {
        object: Object {
            shape: Shape::Rectangle {
                half_size: vector![0.6, 0.6],
            },
            position: Isometry2::new(vector![5.0, 0.0], 0.0),
            linear_velocity: vector![-1.0, 0.0],
            angular_velocity: 0.25 * TAU,
        },
        health: 8,
        time_since_hit: f64::INFINITY,
    });

    loop {
        if input::is_key_pressed(KeyCode::F11) {
            fullscreen ^= true;
            macroquad::window::set_fullscreen(fullscreen);
        }

        utils::update_camera_aspect_ratio(&mut camera);
        camera::set_camera(&camera);

        game.tick(1.0 / 120.0);

        game.draw();

        window::next_frame().await;
    }
}
