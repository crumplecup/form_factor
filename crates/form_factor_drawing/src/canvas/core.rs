//! Core canvas state and error types

use crate::{LayerManager, LayerType, Shape, ToolMode};
use derive_getters::Getters;
use egui::{Color32, Pos2, Stroke};
use form_factor_core::{FieldDefinition, FormTemplate};
use serde::{Deserialize, Serialize};

/// Default zoom level for new canvases
pub(super) fn default_zoom_level() -> f32 {
    5.0
}

/// Kinds of errors that can occur in canvas operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanvasErrorKind {
    /// Failed to load image file
    ImageLoad(String),
    /// Failed to read file from disk
    FileRead(String),
    /// Failed to write file to disk
    FileWrite(String),
    /// Failed to serialize data to JSON
    Serialization(String),
    /// Failed to deserialize data from JSON
    Deserialization(String),
    /// Operation requires a form image but none is loaded
    NoFormImageLoaded,
    /// Text detection operation failed
    TextDetection(String),
    /// Logo detection operation failed
    LogoDetection(String),
    /// No recent projects found
    NoRecentProjects,
    /// OCR text extraction failed
    OCRFailed(String),
}

impl std::fmt::Display for CanvasErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CanvasErrorKind::ImageLoad(msg) => write!(f, "Failed to load image: {}", msg),
            CanvasErrorKind::FileRead(msg) => write!(f, "Failed to read file: {}", msg),
            CanvasErrorKind::FileWrite(msg) => write!(f, "Failed to write file: {}", msg),
            CanvasErrorKind::Serialization(msg) => write!(f, "Failed to serialize data: {}", msg),
            CanvasErrorKind::Deserialization(msg) => {
                write!(f, "Failed to deserialize data: {}", msg)
            }
            CanvasErrorKind::NoFormImageLoaded => write!(f, "No form image loaded"),
            CanvasErrorKind::TextDetection(msg) => write!(f, "Text detection failed: {}", msg),
            CanvasErrorKind::LogoDetection(msg) => write!(f, "Logo detection failed: {}", msg),
            CanvasErrorKind::NoRecentProjects => write!(f, "No recent projects found"),
            CanvasErrorKind::OCRFailed(msg) => write!(f, "OCR text extraction failed: {}", msg),
        }
    }
}

/// Error type for canvas operations
#[derive(Debug, Clone)]
pub struct CanvasError {
    /// The kind of error that occurred
    pub kind: CanvasErrorKind,
    /// Line number where the error was created
    pub line: u32,
    /// File where the error was created
    pub file: &'static str,
}

impl CanvasError {
    /// Create a new canvas error
    pub fn new(kind: CanvasErrorKind, line: u32, file: &'static str) -> Self {
        Self { kind, line, file }
    }
}

impl std::fmt::Display for CanvasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Canvas error: {} at line {} in {}",
            self.kind, self.line, self.file
        )
    }
}

impl std::error::Error for CanvasError {}

/// Canvas interaction state
///
/// Represents the current user interaction mode with the canvas.
/// This state machine prevents invalid state combinations (e.g., drawing while rotating).
#[derive(Debug, Clone, Default)]
pub(super) enum CanvasState {
    /// No active interaction
    #[default]
    Idle,
    /// User is actively drawing a new shape
    Drawing {
        /// Starting position of the shape
        start: Pos2,
        /// Current end position (for rectangles/circles)
        current_end: Option<Pos2>,
        /// Points being drawn (for polygons)
        points: Vec<Pos2>,
    },
    /// User is dragging a vertex in Edit mode
    DraggingVertex {
        /// Index of the vertex being dragged
        vertex_index: usize,
    },
    /// User is rotating a shape in Rotate mode
    Rotating {
        /// Starting angle of rotation in radians
        start_angle: f32,
        /// Center point of rotation
        center: Option<Pos2>,
    },
}

/// Detection sub-type for filtering detections layer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionSubtype {
    /// Logo detections
    Logos,
    /// Text detections
    Text,
}

/// Template editing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TemplateMode {
    /// No template editing
    #[default]
    None,
    /// Creating a new template
    Creating,
    /// Editing an existing template
    Editing,
    /// Viewing template as overlay (read-only)
    Viewing,
}

/// Instance data entry mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InstanceMode {
    /// No instance interaction
    #[default]
    None,
    /// Filling in form data
    Filling,
    /// Viewing instance data (read-only)
    Viewing,
}

/// Drawing canvas state
#[derive(Clone, Serialize, Deserialize, Getters)]
pub struct DrawingCanvas {
    /// File format version (for migration compatibility)
    #[serde(default)]
    pub(super) version: u32,
    /// Project name
    pub(super) project_name: String,
    /// All completed shapes
    pub(super) shapes: Vec<Shape>,
    /// Detected text regions
    pub(super) detections: Vec<Shape>,
    /// Currently active tool
    pub(super) current_tool: ToolMode,
    /// Layer management
    pub(super) layer_manager: LayerManager,
    /// Path to the loaded form image (for serialization)
    pub(super) form_image_path: Option<String>,
    /// Template ID for this canvas (if associated with a template)
    pub(super) template_id: Option<String>,

    // Template and instance state (not serialized for now - will be handled separately)
    /// Current template being created or edited
    #[serde(skip)]
    pub(super) current_template: Option<crate::DrawingTemplateBuilder>,
    /// Current template editing mode
    #[serde(skip)]
    #[serde(default)]
    pub(super) template_mode: TemplateMode,
    
    /// Current instance being filled or viewed
    #[serde(skip)]
    pub(super) current_instance: Option<crate::DrawingInstance>,
    /// Current instance mode
    #[serde(skip)]
    #[serde(default)]
    pub(super) instance_mode: InstanceMode,
    
    /// Selected field index (when Template or Instance layer is active)
    #[serde(skip)]
    pub(super) selected_field: Option<usize>,

    // Interaction state (not serialized)
    /// Current user interaction state (drawing, rotating, etc.)
    #[serde(skip)]
    #[serde(default)]
    pub(super) state: CanvasState,

    // Selection state (not serialized)
    #[serde(skip)]
    pub(super) selected_shape: Option<usize>,
    /// Currently selected layer type
    #[serde(skip)]
    pub(super) selected_layer: Option<LayerType>,
    #[serde(skip)]
    pub(super) show_properties: bool,
    #[serde(skip)]
    pub(super) focus_name_field: bool,
    /// Whether the project name is currently being edited
    #[serde(skip)]
    pub(super) editing_project_name: bool,
    /// Whether the Detections layer dropdown is expanded
    #[serde(skip)]
    pub(super) detections_expanded: bool,
    /// Selected detection sub-type (Logos or Text)
    #[serde(skip)]
    pub(super) selected_detection_subtype: Option<DetectionSubtype>,

    // Form image state (not serialized)
    #[serde(skip)]
    pub(super) form_image: Option<egui::TextureHandle>,
    #[serde(skip)]
    pub(super) form_image_size: Option<egui::Vec2>,
    #[serde(skip)]
    pub(super) pending_image_load: Option<String>,

    // Zoom and pan state
    /// Current zoom level for the canvas
    #[serde(default = "default_zoom_level")]
    pub(super) zoom_level: f32,
    /// Current pan offset for the canvas view
    #[serde(default)]
    pub(super) pan_offset: egui::Vec2,

    // Settings state (not serialized)
    #[serde(skip)]
    pub(super) show_settings: bool,
    #[serde(skip)]
    pub(super) zoom_sensitivity: f32,
    #[serde(skip)]
    pub(super) grid_spacing_horizontal: f32,
    #[serde(skip)]
    pub(super) grid_spacing_vertical: f32,
    /// Rotation angle of the grid overlay in radians
    #[serde(default)]
    pub(super) grid_rotation_angle: f32,

    // Form image rotation
    /// Rotation angle of the form image in radians
    #[serde(default)]
    pub(super) form_image_rotation: f32,

    // Style settings
    /// Stroke style for drawing shapes
    pub(super) stroke: Stroke,
    /// Fill color for drawing shapes
    pub(super) fill_color: Color32,
}

impl Default for DrawingCanvas {
    fn default() -> Self {
        Self {
            version: 1, // Version 1 = legacy single-page format
            project_name: String::from("Untitled"),
            shapes: Vec::new(),
            detections: Vec::new(),
            current_tool: ToolMode::default(),
            layer_manager: LayerManager::new(),
            form_image_path: None,
            template_id: None,
            current_template: None,
            template_mode: TemplateMode::default(),
            current_instance: None,
            instance_mode: InstanceMode::default(),
            selected_field: None,
            state: CanvasState::default(),
            selected_shape: None,
            selected_layer: None,
            show_properties: false,
            focus_name_field: false,
            editing_project_name: false,
            detections_expanded: false,
            selected_detection_subtype: None,
            form_image: None,
            form_image_size: None,
            pending_image_load: None,
            zoom_level: 5.0,
            pan_offset: egui::Vec2::ZERO,
            show_settings: false,
            zoom_sensitivity: 5.0,
            grid_spacing_horizontal: 10.0,
            grid_spacing_vertical: 10.0,
            grid_rotation_angle: 0.0,
            form_image_rotation: 0.0,
            stroke: Stroke::new(2.0, Color32::from_rgb(0, 120, 215)),
            fill_color: Color32::from_rgba_premultiplied(0, 120, 215, 30),
        }
    }
}

impl std::fmt::Debug for DrawingCanvas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DrawingCanvas")
            .field("shapes", &self.shapes)
            .field("detections", &self.detections)
            .field("current_tool", &self.current_tool)
            .field("layer_manager", &self.layer_manager)
            .field("form_image_path", &self.form_image_path)
            .field("form_image_loaded", &self.form_image.is_some())
            .field("form_image_size", &self.form_image_size)
            .field("selected_shape", &self.selected_shape)
            .field("stroke", &self.stroke)
            .field("fill_color", &self.fill_color)
            .finish()
    }
}

impl DrawingCanvas {
    /// Create a new drawing canvas
    pub fn new() -> Self {
        Self::default()
    }

    // Setter methods for externally mutated fields

    /// Set the currently selected layer
    pub fn set_selected_layer(&mut self, layer: Option<LayerType>) {
        self.selected_layer = layer;
    }

    /// Set whether the project name is being edited
    pub fn set_editing_project_name(&mut self, editing: bool) {
        self.editing_project_name = editing;
    }

    /// Set the project name
    pub fn set_project_name(&mut self, name: impl Into<String>) {
        self.project_name = name.into();
    }

    /// Set the template ID for this canvas
    pub fn set_template_id(&mut self, template_id: Option<String>) {
        self.template_id = template_id;
    }

    /// Get a mutable reference to the layer manager
    pub fn layer_manager_mut(&mut self) -> &mut LayerManager {
        &mut self.layer_manager
    }

    // Internal helper methods for module communication

    /// Set the interaction state (for use within canvas module)
    pub(super) fn set_state(&mut self, state: CanvasState) {
        self.state = state;
    }

    /// Get a mutable reference to the state (for use within canvas module)
    pub(super) fn state_mut(&mut self) -> &mut CanvasState {
        &mut self.state
    }

    /// Set the selected shape (for use within canvas module)
    pub(super) fn set_selected_shape(&mut self, shape: Option<usize>) {
        self.selected_shape = shape;
    }

    /// Set show properties flag (for use within canvas module)
    pub(super) fn set_show_properties(&mut self, show: bool) {
        self.show_properties = show;
    }

    /// Set focus name field flag (for use within canvas module)
    pub(super) fn set_focus_name_field(&mut self, focus: bool) {
        self.focus_name_field = focus;
    }

    /// Add a shape to the shapes vector (for use within canvas module)
    pub(super) fn add_shape(&mut self, shape: Shape) {
        self.shapes.push(shape);
    }

    /// Get a mutable reference to the shapes vector (for use within canvas module)
    pub(super) fn shapes_mut(&mut self) -> &mut Vec<Shape> {
        &mut self.shapes
    }

    /// Set the grid rotation angle (for use within canvas module)
    pub(super) fn set_grid_rotation_angle(&mut self, angle: f32) {
        self.grid_rotation_angle = angle;
    }

    /// Set the form image rotation (for use within canvas module)
    pub(super) fn set_form_image_rotation(&mut self, angle: f32) {
        self.form_image_rotation = angle;
    }

    /// Undo the last shape addition (removes the most recently added shape)
    pub fn undo(&mut self) {
        self.shapes.pop();
    }

    /// Get the number of shapes on the canvas
    pub fn shape_count(&self) -> usize {
        self.shapes.len()
    }

    /// Get the number of text detections on the canvas
    pub fn text_detection_count(&self) -> usize {
        self.detections
            .iter()
            .filter(|shape| match shape {
                Shape::Rectangle(rect) => rect.name.starts_with("Text Region"),
                _ => false,
            })
            .count()
    }

    /// Get the number of logo detections on the canvas
    pub fn logo_detection_count(&self) -> usize {
        self.detections
            .iter()
            .filter(|shape| match shape {
                Shape::Rectangle(rect) => rect.name.starts_with("Logo:"),
                _ => false,
            })
            .count()
    }

    /// Toggle the detections layer dropdown expansion state
    pub fn toggle_detections_expanded(&mut self) {
        self.detections_expanded = !self.detections_expanded;
    }

    /// Check if the detections layer dropdown is expanded
    pub fn is_detections_expanded(&self) -> bool {
        self.detections_expanded
    }

    /// Set the selected detection sub-type
    pub fn set_selected_detection_subtype(&mut self, subtype: Option<DetectionSubtype>) {
        self.selected_detection_subtype = subtype;
    }

    /// Set the zoom level
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom_level = zoom.clamp(0.1, 100.0); // Clamp between 0.1 and 100
    }

    /// Set the pan offset
    pub fn set_pan_offset(&mut self, x: f32, y: f32) {
        self.pan_offset = egui::Vec2::new(x, y);
    }

    /// Set the current tool mode
    pub fn set_tool(&mut self, tool: ToolMode) {
        self.current_tool = tool;
    }

    /// Snap a position to the nearest field edge if within threshold
    ///
    /// Checks if the given position is within the snap threshold distance from any
    /// field edge. If so, returns the snapped position. Otherwise returns the original position.
    ///
    /// # Arguments
    ///
    /// * `pos` - Position to snap (in image pixel coordinates)
    /// * `fields` - Field definitions to snap to
    /// * `threshold` - Snap threshold distance in pixels (default: 10.0)
    ///
    /// # Returns
    ///
    /// The snapped position if within threshold, otherwise the original position
    pub fn snap_to_field(&self, pos: Pos2, fields: &[FieldDefinition], threshold: f32) -> Pos2 {
        let mut snapped_x = pos.x;
        let mut snapped_y = pos.y;
        let mut min_x_dist = threshold;
        let mut min_y_dist = threshold;

        // Check each field for snapping opportunities
        for field in fields {
            let bounds = &field.bounds;

            // Check horizontal edges (top and bottom)
            let y_top_dist = (pos.y - bounds.y).abs();
            let y_bottom_dist = (pos.y - (bounds.y + bounds.height)).abs();

            if y_top_dist < min_y_dist {
                min_y_dist = y_top_dist;
                snapped_y = bounds.y;
            }
            if y_bottom_dist < min_y_dist {
                min_y_dist = y_bottom_dist;
                snapped_y = bounds.y + bounds.height;
            }

            // Check vertical edges (left and right)
            let x_left_dist = (pos.x - bounds.x).abs();
            let x_right_dist = (pos.x - (bounds.x + bounds.width)).abs();

            if x_left_dist < min_x_dist {
                min_x_dist = x_left_dist;
                snapped_x = bounds.x;
            }
            if x_right_dist < min_x_dist {
                min_x_dist = x_right_dist;
                snapped_x = bounds.x + bounds.width;
            }
        }

        Pos2::new(snapped_x, snapped_y)
    }

    // Testing helper methods
    // These are public to allow integration tests to verify coordinate system consistency

    /// Add a shape directly to the canvas (for testing coordinate transformations)
    #[doc(hidden)]
    pub fn test_add_shape(&mut self, shape: Shape) {
        self.shapes.push(shape);
    }

    /// Add a detection directly to the canvas (for testing coordinate transformations)
    #[doc(hidden)]
    pub fn test_add_detection(&mut self, detection: Shape) {
        self.detections.push(detection);
    }

    /// Map a detection from image pixel coordinates to canvas coordinates (for testing)
    #[doc(hidden)]
    pub fn test_map_detection_to_canvas(
        &self,
        detection: &Shape,
        scale: f32,
        image_offset: egui::Pos2,
    ) -> Shape {
        self.map_detection_to_canvas(detection, scale, image_offset)
    }

    // Template and Instance Mode Management

    /// Start creating a new template
    pub fn start_template_creation(&mut self, template_id: impl Into<String>, template_name: impl Into<String>) {
        self.current_template = Some(crate::DrawingTemplateBuilder::default()
            .id(template_id)
            .name(template_name)
            .version("1.0.0"));
        self.template_mode = TemplateMode::Creating;
        self.selected_layer = Some(LayerType::Template);
        self.selected_field = None;
    }

    /// Load a template for editing
    pub fn load_template_for_editing(&mut self, template: &crate::DrawingTemplate) {
        // Convert template to builder (we'll need to add a to_builder() method)
        // For now, create a new builder with the template's data
        let builder = crate::DrawingTemplateBuilder::default()
            .id(template.id().to_string())
            .name(template.name().to_string())
            .version(template.version().to_string());
        
        // TODO: Copy pages and fields from template to builder
        // This requires adding a to_builder() or similar method on DrawingTemplate
        
        self.current_template = Some(builder);
        self.template_mode = TemplateMode::Editing;
        self.selected_layer = Some(LayerType::Template);
        self.selected_field = None;
    }

    /// Load a template as read-only overlay
    pub fn load_template_overlay(&mut self, template: &crate::DrawingTemplate) {
        // Similar to editing but read-only
        let builder = crate::DrawingTemplateBuilder::default()
            .id(template.id().to_string())
            .name(template.name().to_string())
            .version(template.version().to_string());
        
        self.current_template = Some(builder);
        self.template_mode = TemplateMode::Viewing;
        self.selected_layer = Some(LayerType::Template);
        self.selected_field = None;
    }

    /// Exit template mode
    pub fn exit_template_mode(&mut self) {
        self.current_template = None;
        self.template_mode = TemplateMode::None;
        self.selected_field = None;
    }

    /// Start filling an instance from a template
    pub fn start_instance_filling(&mut self, template: &crate::DrawingTemplate) {
        let instance = crate::DrawingInstance::from_template(
            template.id().to_string(),
            template.page_count(),
        );
        
        self.current_instance = Some(instance);
        self.instance_mode = InstanceMode::Filling;
        self.selected_layer = Some(LayerType::Instance);
        self.selected_field = None;
    }

    /// Load an instance for viewing
    pub fn load_instance_for_viewing(&mut self, instance: crate::DrawingInstance) {
        self.current_instance = Some(instance);
        self.instance_mode = InstanceMode::Viewing;
        self.selected_layer = Some(LayerType::Instance);
        self.selected_field = None;
    }

    /// Exit instance mode
    pub fn exit_instance_mode(&mut self) {
        self.current_instance = None;
        self.instance_mode = InstanceMode::None;
        self.selected_field = None;
    }

    /// Get the current template mutably (if any)
    pub fn current_template_mut(&mut self) -> Option<&mut crate::DrawingTemplateBuilder> {
        self.current_template.as_mut()
    }

    /// Get the current instance mutably (if any)
    pub fn current_instance_mut(&mut self) -> Option<&mut crate::DrawingInstance> {
        self.current_instance.as_mut()
    }

    /// Set the currently selected field
    pub fn set_selected_field(&mut self, field: Option<usize>) {
        self.selected_field = field;
    }
}
