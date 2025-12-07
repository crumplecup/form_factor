//! Drawing canvas with interactive annotation tools
//!
//! This crate provides the DrawingCanvas, shapes, layers, and tool management.
//! It depends on form_factor_core for the core traits.
//!
//! # Form Templates and Instances
//!
//! - The `template` module provides concrete implementations of the FormTemplate
//!   trait, template storage/registry, and builder patterns for creating templates.
//! - The `instance` module provides concrete implementations of the FormInstance
//!   trait, multi-page support, and instance management.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod app_mode;
mod canvas;
mod error;
pub mod instance;
pub mod instance_ui;
mod layer;
mod mode_switcher;
mod recent_projects;
mod shape;
pub mod template;
pub mod template_ui;
mod tool;

pub use app_mode::{AppMode, AppState};
pub use canvas::{CanvasError, CanvasErrorKind, CanvasState, DetectionSubtype, DrawingCanvas};
pub use error::{FormError, FormErrorKind};
pub use instance::{
    DrawingInstance, FormPage, InstanceError, InstanceErrorKind, LEGACY_TEMPLATE_ID, ProjectFormat,
    migrate_canvas_to_instance,
};
pub use instance_ui::{
    DataEntryAction, DataEntryPanel, InstanceManagerAction, InstanceManagerPanel,
};
pub use layer::{Layer, LayerError, LayerManager, LayerType};
pub use mode_switcher::ModeSwitcher;
pub use recent_projects::RecentProjects;
pub use shape::{
    Circle, CircleBuilder, PolygonShape, Rectangle, Shape, ShapeError, ShapeErrorKind,
};
pub use template::{
    DrawingTemplate, DrawingTemplateBuilder, TemplateError, TemplateErrorKind, TemplatePage,
    TemplatePageBuilder, TemplateRegistry,
};
pub use template_ui::{
    EditorAction, EditorMode, FieldPropertiesPanel, ManagerAction, PropertiesAction,
    TemplateEditorPanel, TemplateEditorState, TemplateManagerPanel, TemplateManagerState,
    TemplateSnapshot, TemplateValidator, ValidationError,
};
pub use tool::ToolMode;
