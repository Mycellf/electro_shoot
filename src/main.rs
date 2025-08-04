pub mod enemy;
pub mod game;
pub mod object;
pub mod projectile;
pub mod shape;
pub mod utils;

use macroquad::{
    camera::{self, Camera2D},
    input::{self, KeyCode},
    window::{self, Conf},
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
    let mut fullscreen = START_IN_FULLSCREEN;

    let screen_height = 10.0;
    let mut camera = Camera2D {
        zoom: [-2.0 / screen_height as f32; 2].into(),
        ..Default::default()
    };

    loop {
        if input::is_key_pressed(KeyCode::F11) {
            fullscreen ^= true;
            macroquad::window::set_fullscreen(fullscreen);
        }

        utils::update_camera_aspect_ratio(&mut camera);
        camera::set_camera(&camera);

        window::next_frame().await;
    }
}
