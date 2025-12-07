//! Object event handlers

use form_factor::{DrawingCanvas, LayerType};
use tracing::instrument;

/// Object event handler
pub struct ObjectEventHandler;

impl ObjectEventHandler {
    /// Handle object delete requested
    #[instrument(skip(canvas), fields(layer_type = ?layer_type, object_index))]
    pub fn handle_delete_requested(
        canvas: &mut DrawingCanvas,
        layer_type: &LayerType,
        object_index: usize,
    ) {
        tracing::info!(?layer_type, object_index, "Deleting object from layer");

        match layer_type {
            LayerType::Shapes => {
                canvas.delete_shape(object_index);
            }
            LayerType::Detections => {
                canvas.delete_detection(object_index);
            }
            _ => {
                tracing::warn!(?layer_type, "Layer does not support object deletion");
            }
        }
    }

    /// Handle object visibility changed
    #[instrument(skip(canvas), fields(layer_type = ?layer_type, object_index, visible))]
    pub fn handle_visibility_changed(
        canvas: &mut DrawingCanvas,
        layer_type: &LayerType,
        object_index: usize,
        visible: bool,
    ) {
        tracing::info!(
            ?layer_type,
            object_index,
            visible,
            "Changing object visibility"
        );

        match layer_type {
            LayerType::Shapes => {
                if let Err(e) = canvas.set_shape_visibility(object_index, visible) {
                    tracing::error!(error = %e, "Failed to set shape visibility");
                }
            }
            LayerType::Detections => {
                if let Err(e) = canvas.set_detection_visibility(object_index, visible) {
                    tracing::error!(error = %e, "Failed to set detection visibility");
                }
            }
            _ => {
                tracing::warn!(?layer_type, "Layer does not support object visibility");
            }
        }
    }

    /// Handle OCR object delete requested
    #[cfg(feature = "plugin-layers")]
    #[instrument(skip(canvas), fields(index))]
    pub fn handle_ocr_delete_requested(canvas: &mut DrawingCanvas, index: usize) {
        tracing::info!(index, "Deleting OCR detection");
        canvas.delete_ocr_detection(index);
    }

    /// Handle OCR object visibility changed
    #[cfg(feature = "plugin-layers")]
    #[instrument(skip(canvas), fields(index, visible))]
    pub fn handle_ocr_visibility_changed(
        canvas: &mut DrawingCanvas,
        index: usize,
        visible: bool,
    ) {
        tracing::info!(index, visible, "Changing OCR detection visibility");

        if let Err(e) = canvas.set_ocr_detection_visibility(index, visible) {
            tracing::error!(error = %e, "Failed to change OCR detection visibility");
        }
    }
}
