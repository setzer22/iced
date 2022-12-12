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

/// A transformation consisting only of translation and uniform scaling
/// operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TranslateScale {
    translation: Vector,
    scale: f32,
}

impl TranslateScale {
    /// Returns the identity transformation.
    pub fn identity() -> Self {
        Self {
            translation: Vector::new(0.0, 0.0),
            scale: 1.0,
        }
    }

    /// Returns a new transformation, translated by the given vector
    pub fn translated(&self, translation: Vector) -> Self {
        Self {
            translation: self.translation + translation,
            scale: self.scale,
        }
    }

    /// Returns a new transformation, scaled by the given amount
    pub fn scaled(&self, scale: f32) -> Self {
        let new_scale = self.scale * scale;
        Self {
            translation: self.translation * scale,
            scale: new_scale,
        }
    }

    /// Applies the scaling and translation of this transformation to the given
    /// `point`.
    pub fn transform_point(&self, point: Point) -> Point {
        Point::new(point.x * self.scale, point.y * self.scale) + self.translation
    }

    /// Applies the scaling of this transformation to the given `scalar`.
    /// Translation is ignored.
    pub fn transform_scalar(&self, s: f32) -> f32 {
        s * self.scale
    }

    /// Applies the scaling and translation of this transformation to the given
    /// `rectangle`. The rectangles's dimensions may be set to infinity.
    pub fn transform_rectangle(&self, rectangle: Rectangle) -> Rectangle {
        let top_left =
            self.transform_point(Point::new(rectangle.x, rectangle.y));

        Rectangle {
            x: top_left.x,
            y: top_left.y,
            width: rectangle.width * self.scale,
            height: rectangle.height * self.scale,
        }
    }
}
