//! Organize rendering primitives into a flattened list of layers.
mod image;
mod quad;
mod text;

pub mod mesh;

pub use image::Image;
pub use mesh::Mesh;
pub use quad::Quad;
pub use text::Text;

use crate::{alignment, Transformation};
use crate::{
    Background, Font, Point, Primitive, Rectangle, Size, Vector, Viewport,
};

/// A group of primitives that should be clipped together.
#[derive(Debug)]
pub struct Layer<'a> {
    /// The clipping bounds of the [`Layer`].
    pub bounds: Rectangle,

    /// The quads of the [`Layer`].
    pub quads: Vec<Quad>,

    /// The triangle meshes of the [`Layer`].
    pub meshes: Vec<Mesh<'a>>,

    /// The text of the [`Layer`].
    pub text: Vec<Text<'a>>,

    /// The images of the [`Layer`].
    pub images: Vec<Image>,
}

impl<'a> Layer<'a> {
    /// Creates a new [`Layer`] with the given clipping bounds.
    pub fn new(bounds: Rectangle) -> Self {
        Self {
            bounds,
            quads: Vec::new(),
            meshes: Vec::new(),
            text: Vec::new(),
            images: Vec::new(),
        }
    }

    /// Creates a new [`Layer`] for the provided overlay text.
    ///
    /// This can be useful for displaying debug information.
    pub fn overlay(lines: &'a [impl AsRef<str>], viewport: &Viewport) -> Self {
        let mut overlay =
            Layer::new(Rectangle::with_size(viewport.logical_size()));

        for (i, line) in lines.iter().enumerate() {
            let text = Text {
                content: line.as_ref(),
                bounds: Rectangle::new(
                    Point::new(11.0, 11.0 + 25.0 * i as f32),
                    Size::INFINITY,
                ),
                color: [0.9, 0.9, 0.9, 1.0],
                size: 20.0,
                font: Font::Default,
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
            };

            overlay.text.push(text);

            overlay.text.push(Text {
                bounds: text.bounds + Vector::new(-1.0, -1.0),
                color: [0.0, 0.0, 0.0, 1.0],
                ..text
            });
        }

        overlay
    }

    /// Distributes the given [`Primitive`] and generates a list of layers based
    /// on its contents.
    pub fn generate(
        primitives: &'a [Primitive],
        viewport: &Viewport,
    ) -> Vec<Self> {
        let first_layer =
            Layer::new(Rectangle::with_size(viewport.logical_size()));

        let mut layers = vec![first_layer];

        for primitive in primitives {
            Self::process_primitive(
                &mut layers,
                Transformation::identity(),
                primitive,
                0,
            );
        }

        layers
    }

    fn process_primitive(
        layers: &mut Vec<Self>,
        transformation: Transformation,
        primitive: &'a Primitive,
        current_layer: usize,
    ) {
        match primitive {
            Primitive::None => {}
            Primitive::Group { primitives } => {
                // TODO: Inspect a bit and regroup (?)
                for primitive in primitives {
                    Self::process_primitive(
                        layers,
                        transformation,
                        primitive,
                        current_layer,
                    )
                }
            }
            Primitive::Text {
                content,
                bounds,
                size,
                color,
                font,
                horizontal_alignment,
                vertical_alignment,
            } => {
                let layer = &mut layers[current_layer];

                layer.text.push(Text {
                    content,
                    bounds: transformation.transform_rectangle(*bounds),
                    size: transformation.transform_scalar(*size),
                    color: color.into_linear(),
                    font: *font,
                    horizontal_alignment: *horizontal_alignment,
                    vertical_alignment: *vertical_alignment,
                });
            }
            Primitive::Quad {
                bounds,
                background,
                border_radius,
                border_width,
                border_color,
            } => {
                let layer = &mut layers[current_layer];

                // TODO: Move some of these computations to the GPU (?)
                let new_bounds = transformation.transform_rectangle(*bounds);

                layer.quads.push(Quad {
                    position: [new_bounds.x, new_bounds.y],
                    size: [new_bounds.width, new_bounds.height],
                    color: match background {
                        Background::Color(color) => color.into_linear(),
                    },
                    border_radius: transformation.transform_scalar(*border_radius),
                    border_width: transformation.transform_scalar(*border_width),
                    border_color: border_color.into_linear(),
                });
            }
            Primitive::Mesh2D {
                buffers,
                size,
                style,
            } => {
                let layer = &mut layers[current_layer];

                // TODO: Can't apply scale to a mesh...
                let origin =
                    transformation.transform_point(Point::new(0.0, 0.0));

                let bounds =
                    Rectangle::new(Point::new(origin.x, origin.y), *size);

                // Only draw visible content
                if let Some(clip_bounds) = layer.bounds.intersection(&bounds) {
                    layer.meshes.push(Mesh {
                        origin,
                        buffers,
                        clip_bounds,
                        style,
                    });
                }
            }
            Primitive::Clip { bounds, content } => {
                let layer = &mut layers[current_layer];
                let transformed_bounds =
                    transformation.transform_rectangle(*bounds);

                // Only draw visible content
                if let Some(clip_bounds) =
                    layer.bounds.intersection(&transformed_bounds)
                {
                    let clip_layer = Layer::new(clip_bounds);
                    layers.push(clip_layer);

                    Self::process_primitive(
                        layers,
                        transformation,
                        content,
                        layers.len() - 1,
                    );
                }
            }
            Primitive::Translate {
                translation: new_translation,
                content,
            } => {
                Self::process_primitive(
                    layers,
                    transformation
                        .translated(new_translation.x, new_translation.y),
                    content,
                    current_layer,
                );
            }
            Primitive::Scale { scale, content } => {
                Self::process_primitive(
                    layers,
                    transformation.scaled(*scale, *scale),
                    content,
                    current_layer,
                );
            }
            Primitive::Cached { cache } => {
                Self::process_primitive(
                    layers,
                    transformation,
                    cache,
                    current_layer,
                );
            }
            Primitive::Image { handle, bounds } => {
                let layer = &mut layers[current_layer];

                layer.images.push(Image::Raster {
                    handle: handle.clone(),
                    bounds: transformation.transform_rectangle(*bounds),
                });
            }
            Primitive::Svg { handle, bounds } => {
                let layer = &mut layers[current_layer];

                layer.images.push(Image::Vector {
                    handle: handle.clone(),
                    bounds: transformation.transform_rectangle(*bounds),
                });
            }
        }
    }
}
