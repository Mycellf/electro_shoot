pub mod enemy;
pub mod game;
pub mod object;
pub mod particle;
pub mod projectile;
pub mod shape;
pub mod turret;
pub mod utils;

use std::f64::consts::TAU;

use macroquad::{
    camera::{self, Camera2D},
    input::{self, KeyCode},
    window::{self, Conf},
};
use nalgebra::{Isometry2, vector};

use crate::{
    enemy::{ENEMY_KINDS, Enemy},
    game::Game,
};

const START_IN_FULLSCREEN: bool = true;

fn config() -> Conf {
    Conf {
        window_title: "Electro Shoot".to_owned(),
        fullscreen: START_IN_FULLSCREEN,
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    macroquad::rand::srand(1234980);

    let mut fullscreen = START_IN_FULLSCREEN;

    let screen_height = 36.0;
    let mut camera = Camera2D {
        zoom: [-2.0 / screen_height as f32; 2].into(),
        ..Default::default()
    };

    let mut game = Game::default();

    game.enemies.insert(Enemy::new(
        Isometry2::new(vector![25.0, 0.0], 0.5 * TAU),
        &ENEMY_KINDS[0],
    ));

    loop {
        if input::is_key_pressed(KeyCode::F11) {
            fullscreen ^= true;
            macroquad::window::set_fullscreen(fullscreen);
        }

        utils::update_camera_aspect_ratio(&mut camera);
        camera::set_camera(&camera);

        game.tick_input(macroquad::time::get_frame_time() as f64);

        game.tick(&mut camera, 1.0 / 120.0);

        game.draw();

        window::next_frame().await;
    }
}
