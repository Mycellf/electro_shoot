pub mod shape;
pub mod utils;

use std::f64::consts::TAU;

use macroquad::{
    camera::{self, Camera2D},
    color::colors,
    input::{self, KeyCode},
    window::{self, Conf},
};
use nalgebra::{Isometry2, UnitComplex, vector};

use crate::shape::Shape;

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

    let shape_a = Shape::Rectangle {
        half_size: vector![2.5, 1.0],
    };

    let position_a = Isometry2::new(vector![1.0, -1.5], 0.1 * TAU);

    let shape_b = Shape::Point;

    let mut position_b = Isometry2::new(vector![0.0, 0.0], 0.0);

    loop {
        if input::is_key_pressed(KeyCode::F11) {
            fullscreen ^= true;
            macroquad::window::set_fullscreen(fullscreen);
        }

        utils::update_camera_aspect_ratio(&mut camera);

        position_b.translation.vector = utils::mouse_position(&camera);

        let scroll = input::mouse_wheel().1.clamp(-1.0, 1.0) as f64;
        if scroll != 0.0 {
            position_b.append_rotation_wrt_center_mut(&UnitComplex::new(scroll * 0.005 * TAU));
        }

        camera::set_camera(&camera);

        let offset = position_a.inverse() * position_b;

        let maybe_colliding = shape_a
            .bounding_circle()
            .is_colliding(&shape_b.bounding_circle(), offset.translation.vector);
        let colliding = shape_a.is_colliding(&shape_b, offset);

        let color = match (maybe_colliding, colliding) {
            (true, true) => colors::RED,
            (true, false) => colors::BLUE,
            (false, true) => colors::MAGENTA, // should be unreachable
            (false, false) => colors::GREEN,
        };

        shape_a.draw_outline(position_a, 0.1, color);
        shape_b.draw_outline(position_b, 0.1, color);

        window::next_frame().await;
    }
}
