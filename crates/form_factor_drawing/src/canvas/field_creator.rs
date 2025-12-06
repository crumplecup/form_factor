//! Field creation from canvas shapes
//!
//! Converts canvas drawing shapes into template fields

use crate::{shape::Shape, TemplateError};
use derive_getters::Getters;
use form_factor_core::{FieldBoundsBuilder, FieldDefinition, FieldDefinitionBuilder, FieldType};
use tracing::{debug, instrument};

/// Error kinds for field creation operations
#[derive(Debug, Clone, derive_more::Display)]
pub enum FieldCreatorErrorKind {
    /// Template error occurred during field creation
    #[display("Template error: {}", _0)]
    Template(TemplateError),

    /// Field name is required but was not provided
    #[display("Field name is required")]
    MissingFieldName,

    /// Field type is required but was not provided
    #[display("Field type is required")]
    MissingFieldType,

    /// Builder error occurred
    #[display("Builder error: {}", _0)]
    BuilderError(String),
}

/// Error wrapper for field creation operations
#[derive(Debug, Clone, derive_more::Display, derive_more::Error)]
#[display("FieldCreator: {} at {}:{}", kind, file, line)]
pub struct FieldCreatorError {
    kind: FieldCreatorErrorKind,
    line: u32,
    file: &'static str,
}

impl FieldCreatorError {
    /// Creates a new field creator error
    #[track_caller]
    pub fn new(kind: FieldCreatorErrorKind) -> Self {
        let loc = std::panic::Location::caller();
        Self {
            kind,
            line: loc.line(),
            file: loc.file(),
        }
    }
}

impl From<TemplateError> for FieldCreatorError {
    fn from(err: TemplateError) -> Self {
        Self::new(FieldCreatorErrorKind::Template(err))
    }
}

/// Result type for field creator operations
pub type FieldCreatorResult<T> = Result<T, FieldCreatorError>;

/// Converts canvas shapes into template fields
#[derive(Debug, Clone, Getters, derive_setters::Setters)]
#[setters(prefix = "with_")]
pub struct FieldCreator {
    /// Default field type for new fields
    default_field_type: FieldType,
    /// Counter for auto-generating field names
    field_counter: usize,
}

impl FieldCreator {
    /// Creates a new field creator
    #[instrument]
    pub fn new() -> Self {
        debug!("Creating field creator");
        Self {
            default_field_type: FieldType::FreeText,
            field_counter: 0,
        }
    }

    /// Creates a field from a shape
    #[instrument(skip(self, shape))]
    pub fn create_field(
        &mut self,
        shape: &Shape,
        field_type: Option<FieldType>,
        name: Option<String>,
    ) -> FieldCreatorResult<FieldDefinition> {
        let field_type = field_type.unwrap_or_else(|| self.default_field_type.clone());
        let name = name.unwrap_or_else(|| {
            self.field_counter += 1;
            format!("field_{}", self.field_counter)
        });

        debug!(?field_type, ?name, "Creating field from shape");

        // Calculate bounding box from shape
        let (x, y, width, height) = match shape {
            Shape::Rectangle(rect) => {
                let corners = rect.corners();
                let min_x = corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                let max_x = corners.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
                let min_y = corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                let max_y = corners.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
                (min_x, min_y, max_x - min_x, max_y - min_y)
            }
            Shape::Circle(circle) => {
                let center = circle.center();
                let radius = *circle.radius();
                (
                    center.x - radius,
                    center.y - radius,
                    radius * 2.0,
                    radius * 2.0,
                )
            }
            Shape::Polygon(poly) => {
                let points = poly.to_egui_points();
                let min_x = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
                let min_y = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
                (min_x, min_y, max_x - min_x, max_y - min_y)
            }
        };

        let field_bounds = FieldBoundsBuilder::default()
            .x(x)
            .y(y)
            .width(width)
            .height(height)
            .build()
            .map_err(|e| FieldCreatorError::new(FieldCreatorErrorKind::BuilderError(e.to_string())))?;

        FieldDefinitionBuilder::default()
            .id(name.clone())
            .label(name)
            .field_type(field_type)
            .bounds(field_bounds)
            .page_index(0)
            .build()
            .map_err(|e| FieldCreatorError::new(FieldCreatorErrorKind::BuilderError(e.to_string())))
    }

    /// Creates a field from a shape with page index
    ///
    /// This is a convenience wrapper around `create_field` that sets the page index.
    #[instrument(skip(self, shape))]
    pub fn create_field_for_page(
        &mut self,
        shape: &Shape,
        field_type: Option<FieldType>,
        name: Option<String>,
        page_index: usize,
    ) -> FieldCreatorResult<FieldDefinition> {
        let field_type = field_type.unwrap_or_else(|| self.default_field_type.clone());
        let name = name.unwrap_or_else(|| {
            self.field_counter += 1;
            format!("field_{}", self.field_counter)
        });

        debug!(?field_type, ?name, page_index, "Creating field from shape for page");

        // Calculate bounding box from shape
        let (x, y, width, height) = match shape {
            Shape::Rectangle(rect) => {
                let corners = rect.corners();
                let min_x = corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                let max_x = corners.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
                let min_y = corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                let max_y = corners.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
                (min_x, min_y, max_x - min_x, max_y - min_y)
            }
            Shape::Circle(circle) => {
                let center = circle.center();
                let radius = *circle.radius();
                (
                    center.x - radius,
                    center.y - radius,
                    radius * 2.0,
                    radius * 2.0,
                )
            }
            Shape::Polygon(poly) => {
                let points = poly.to_egui_points();
                let min_x = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
                let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
                let min_y = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
                let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
                (min_x, min_y, max_x - min_x, max_y - min_y)
            }
        };

        let field_bounds = FieldBoundsBuilder::default()
            .x(x)
            .y(y)
            .width(width)
            .height(height)
            .build()
            .map_err(|e| FieldCreatorError::new(FieldCreatorErrorKind::BuilderError(e.to_string())))?;

        FieldDefinitionBuilder::default()
            .id(name.clone())
            .label(name)
            .field_type(field_type)
            .bounds(field_bounds)
            .page_index(page_index)
            .build()
            .map_err(|e| FieldCreatorError::new(FieldCreatorErrorKind::BuilderError(e.to_string())))
    }

    /// Resets the field counter
    pub fn reset_counter(&mut self) {
        self.field_counter = 0;
    }
}

impl Default for FieldCreator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::Rectangle;
    use egui::{Color32, Pos2, Stroke};

    #[test]
    fn test_create_field_from_rectangle() {
        let mut creator = FieldCreator::new();

        let rect = Rectangle::new(
            Pos2::new(10.0, 20.0),
            100.0,
            50.0,
            Color32::RED,
            Stroke::new(1.0, Color32::BLACK),
        )
        .expect("Valid rectangle");

        let shape = Shape::Rectangle(rect);
        let field = creator
            .create_field(&shape, Some(FieldType::FirstName), Some("test_field".to_string()))
            .expect("Valid field");

        assert_eq!(field.id(), "test_field");
        assert_eq!(field.label(), "test_field");
        assert_eq!(*field.field_type(), FieldType::FirstName);
        assert_eq!(*field.bounds().x(), 10.0);
        assert_eq!(*field.bounds().y(), 20.0);
        assert_eq!(*field.bounds().width(), 100.0);
        assert_eq!(*field.bounds().height(), 50.0);
    }

    #[test]
    fn test_auto_generate_field_name() {
        let mut creator = FieldCreator::new();

        let rect = Rectangle::new(
            Pos2::new(10.0, 20.0),
            100.0,
            50.0,
            Color32::RED,
            Stroke::new(1.0, Color32::BLACK),
        )
        .expect("Valid rectangle");

        let shape = Shape::Rectangle(rect);
        let field1 = creator
            .create_field(&shape, None, None)
            .expect("Valid field");
        let field2 = creator
            .create_field(&shape, None, None)
            .expect("Valid field");

        assert_eq!(field1.id(), "field_1");
        assert_eq!(field2.id(), "field_2");
    }
}
