use glam::{Mat4, Vec3};
use iced_native::{Point, Rectangle, Vector};
use std::ops::Mul;

/// A 2D transformation matrix.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transformation(Mat4);

impl Transformation {
    /// Get the identity transformation.
    pub fn identity() -> Transformation {
        Transformation(Mat4::IDENTITY)
    }

    /// Creates an orthographic projection.
    #[rustfmt::skip]
    pub fn orthographic(width: u32, height: u32) -> Transformation {
        Transformation(Mat4::orthographic_rh_gl(
            0.0, width as f32,
            height as f32, 0.0,
            -1.0, 1.0
        ))
    }

    /// Creates a translate transformation.
    pub fn translate(x: f32, y: f32) -> Transformation {
        Transformation(Mat4::from_translation(Vec3::new(x, y, 0.0)))
    }

    /// Returns a new transformation, translated by a certain offset
    pub fn translated(&self, x: f32, y: f32) -> Transformation {
        Transformation(Mat4::from_translation(Vec3::new(x, y, 0.0)) * self.0)
    }

    /// Creates a scale transformation.
    pub fn scale(x: f32, y: f32) -> Transformation {
        Transformation(Mat4::from_scale(Vec3::new(x, y, 1.0)))
    }

    /// Returns a new transformation, translated by a certain offset
    pub fn scaled(&self, x: f32, y: f32) -> Transformation {
        Transformation(Mat4::from_scale(Vec3::new(x, y, 0.0)) * self.0)
    }

    /// Applies this transformation to the given `point`.
    pub fn transform_point(&self, point: Point) -> Point {
        let p = self
            .0
            .transform_point3(glam::Vec3::new(point.x, point.y, 0.0));
        Point::new(p.x, p.y)
    }

    /// Applies this transformation to the given `vector`.
    pub fn transform_vector(&self, vector: Vector) -> Vector {
        let p = self
            .0
            .transform_vector3(glam::Vec3::new(vector.x, vector.y, 0.0));
        Vector::new(p.x, p.y)
    }

    /// Applies this transformation to the given `scalar`. Only scaling is
    /// applied TODO: This is a very awkward API. Let's not use Transformation
    /// in the layer computation.
    pub fn transform_scalar(&self, s: f32) -> f32 {
        self.0.transform_vector3(glam::Vec3::new(s, 0.0, 0.0)).x
    }

    /// Applies this transformation to the given `rectangle`.
    ///
    /// NOTE: This operation is not well-defined when the transformation
    /// contains something other than translation and scaling because a
    /// rectangle is an axis-aligned bounding box, so it can't be rotated.
    pub fn transform_rectangle(&self, rectangle: Rectangle) -> Rectangle {
        let top_left = Point::new(rectangle.x, rectangle.y);
        let bottom_right = Point::new(
            rectangle.x + rectangle.width,
            rectangle.y + rectangle.height,
        );

        let new_top_left = self.transform_point(top_left);
        let new_bottom_right = self.transform_point(bottom_right);

        Rectangle {
            x: new_top_left.x,
            y: new_top_left.y,
            width: new_bottom_right.x - new_top_left.x,
            height: new_bottom_right.y - new_top_left.y,
        }
    }
}

impl Mul for Transformation {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Transformation(self.0 * rhs.0)
    }
}

impl AsRef<[f32; 16]> for Transformation {
    fn as_ref(&self) -> &[f32; 16] {
        self.0.as_ref()
    }
}

impl From<Transformation> for [f32; 16] {
    fn from(t: Transformation) -> [f32; 16] {
        *t.as_ref()
    }
}

impl From<Transformation> for Mat4 {
    fn from(transformation: Transformation) -> Self {
        transformation.0
    }
}
