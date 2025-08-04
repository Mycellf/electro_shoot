use macroquad::{
    color::Color,
    shapes::{self, DrawRectangleParams},
};
use nalgebra::{Isometry2, UnitComplex, Vector2, vector};

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    /// Points will never be marked as colliding with each other
    Point,
    Circle {
        radius: f64,
    },
    Rectangle {
        half_size: Vector2<f64>,
    },
}

impl Shape {
    /// `offset` is the transformation from `self` to `other`
    #[must_use]
    pub fn is_colliding(&self, other: &Self, offset: Isometry2<f64>) -> bool {
        match (self, other) {
            (Shape::Point, Shape::Point) => false,

            (Shape::Point, Shape::Circle { radius }) | (Shape::Circle { radius }, Shape::Point) => {
                circle_point(*radius, offset.translation.vector)
            }

            (Shape::Circle { radius: radius_a }, Shape::Circle { radius: radius_b }) => {
                circle_circle(*radius_a, *radius_b, offset.translation.vector)
            }

            (Shape::Point, Shape::Rectangle { half_size: size }) => {
                rectangle_point(*size, offset.inverse().translation.vector)
            }
            (Shape::Rectangle { half_size: size }, Shape::Point) => {
                rectangle_point(*size, offset.translation.vector)
            }

            (Shape::Circle { radius }, Shape::Rectangle { half_size: size }) => {
                rectangle_circle(*size, *radius, offset.inverse().translation.vector)
            }
            (Shape::Rectangle { half_size: size }, Shape::Circle { radius }) => {
                rectangle_circle(*size, *radius, offset.translation.vector)
            }

            (Shape::Rectangle { half_size: size_a }, Shape::Rectangle { half_size: size_b }) => {
                rectangle_rectangle(*size_a, *size_b, offset)
            }
        }
    }

    pub fn draw_outline(&self, position: Isometry2<f64>, thickness: f64, color: Color) {
        match self {
            Shape::Point => shapes::draw_circle(
                position.translation.x as f32,
                position.translation.y as f32,
                thickness as f32 / 2.0,
                color,
            ),
            Shape::Circle { radius } => shapes::draw_circle_lines(
                position.translation.x as f32,
                position.translation.y as f32,
                (*radius - thickness) as f32,
                thickness as f32,
                color,
            ),
            Shape::Rectangle { half_size } => shapes::draw_rectangle_lines_ex(
                position.translation.x as f32,
                position.translation.y as f32,
                half_size.x as f32 * 2.0,
                half_size.y as f32 * 2.0,
                thickness as f32,
                DrawRectangleParams {
                    offset: [0.5; 2].into(),
                    rotation: position.rotation.angle() as f32,
                    color,
                },
            ),
        }
    }
}

fn circle_point(radius: f64, offset: Vector2<f64>) -> bool {
    offset.magnitude_squared() < radius.powi(2)
}

fn circle_circle(radius_a: f64, radius_b: f64, offset: Vector2<f64>) -> bool {
    offset.magnitude_squared() < (radius_a + radius_b).powi(2)
}

fn rectangle_point(half_size: Vector2<f64>, offset: Vector2<f64>) -> bool {
    offset.x.abs() <= half_size.x && offset.y.abs() <= half_size.y
}

fn rectangle_circle(half_size: Vector2<f64>, radius: f64, offset: Vector2<f64>) -> bool {
    // The rectangle is symmetric about the x and y axis
    let offset = offset.abs();

    if offset.y <= half_size.y {
        offset.x <= half_size.x + radius
    } else if offset.x <= half_size.x {
        offset.y <= half_size.y + radius
    } else {
        circle_point(radius, offset - half_size)
    }
}

fn rectangle_rectangle(
    half_size_a: Vector2<f64>,
    half_size_b: Vector2<f64>,
    offset: Isometry2<f64>,
) -> bool {
    rectangle_rectangle_one_sided(half_size_a, half_size_b, offset)
        && rectangle_rectangle_one_sided(half_size_b, half_size_a, offset.inverse())
}

// If this function returns false, the rectangles are not colliding
//
// If it returns true for both the current inputs and the inverse
// `(half_size_b, half_size_a, offset.inverse())` they are colliding.
fn rectangle_rectangle_one_sided(
    half_size_a: Vector2<f64>,
    half_size_b: Vector2<f64>,
    offset: Isometry2<f64>,
) -> bool {
    let half_size_b = bounding_box_of_rectangle(half_size_b, offset.rotation);

    let offset = offset.translation.vector.abs();
    offset.x <= half_size_a.x + half_size_b.x && offset.y <= half_size_a.y + half_size_b.y
}

fn bounding_box_of_rectangle(half_size: Vector2<f64>, rotation: UnitComplex<f64>) -> Vector2<f64> {
    let a = (rotation * half_size).abs();
    let b = (rotation * vector![half_size.x, -half_size.y]).abs();

    vector![a.x.max(b.x), a.y.max(b.y)]
}
