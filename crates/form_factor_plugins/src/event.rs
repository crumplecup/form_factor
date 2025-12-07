//! Event types for plugin communication.

use std::path::PathBuf;

/// Events that can be sent between plugins and the application.
///
/// Events enable decoupled communication between plugins. Plugins can emit
/// events when something happens and subscribe to events from other plugins.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AppEvent {
    /// Canvas zoom level changed
    CanvasZoomChanged {
        /// New zoom factor
        zoom: f32,
    },

    /// Canvas pan position changed
    CanvasPanChanged {
        /// New X offset
        x: f32,
        /// New Y offset
        y: f32,
    },

    /// A shape was selected
    ShapeSelected {
        /// Index of the selected shape
        index: usize,
    },

    /// Shape selection was cleared
    SelectionCleared,

    /// A layer was selected
    LayerSelected {
        /// Name of the selected layer
        layer_name: String,
    },

    /// Layer visibility changed
    LayerVisibilityChanged {
        /// Name of the layer
        layer_name: String,
        /// Whether the layer is visible
        visible: bool,
    },

    /// Request to clear all objects from a layer
    LayerClearRequested {
        /// Name of the layer to clear
        layer_name: String,
    },

    /// Request to delete a specific object from a layer
    #[cfg(feature = "plugin-layers")]
    ObjectDeleteRequested {
        /// Type of the layer containing the object
        layer_type: form_factor_drawing::LayerType,
        /// Index of the object to delete
        object_index: usize,
    },

    /// Visibility of a specific object changed
    #[cfg(feature = "plugin-layers")]
    ObjectVisibilityChanged {
        /// Type of the layer containing the object
        layer_type: form_factor_drawing::LayerType,
        /// Index of the object
        object_index: usize,
        /// Whether the object is visible
        visible: bool,
    },

    /// A file was opened
    FileOpened {
        /// Path to the opened file
        path: PathBuf,
    },

    /// A file was saved
    FileSaved {
        /// Path where the file was saved
        path: PathBuf,
    },

    /// User requested to open a file
    OpenFileRequested,

    /// User requested to save a file
    SaveFileRequested,

    /// User requested to save file with new name
    SaveAsRequested,

    /// User requested to load an image onto the canvas
    LoadImageRequested,

    /// Text detection was requested
    TextDetectionRequested,

    /// Logo detection was requested
    LogoDetectionRequested,

    /// OCR text extraction was requested
    OcrExtractionRequested,

    /// Request to delete an OCR detection
    #[cfg(feature = "plugin-layers")]
    OcrObjectDeleteRequested {
        /// Index of the OCR detection to delete
        index: usize,
    },

    /// Visibility of an OCR detection changed
    #[cfg(feature = "plugin-layers")]
    OcrObjectVisibilityChanged {
        /// Index of the OCR detection
        index: usize,
        /// Whether the detection is visible
        visible: bool,
    },

    /// Request to clear the canvas image
    CanvasImageClearRequested,

    /// Canvas image visibility changed
    CanvasImageVisibilityChanged {
        /// Whether the canvas image is visible
        visible: bool,
    },

    /// Canvas image lock state changed
    CanvasImageLockChanged {
        /// Whether the canvas image is locked
        locked: bool,
    },

    /// Detection operation has started
    DetectionStarted {
        /// Type of detection starting
        detection_type: String,
    },

    /// Detection operation failed
    DetectionFailed {
        /// Type of detection that failed
        detection_type: String,
        /// Error message
        error: String,
    },

    /// Detection results are available
    DetectionComplete {
        /// Number of detections found
        count: usize,
        /// Type of detection
        detection_type: String,
    },

    /// Detection results with shape data
    DetectionResultsReady {
        /// Type of detection
        detection_type: String,
        /// Serialized shape data (JSON)
        shapes_json: String,
    },

    /// OCR extraction completed
    OcrComplete {
        /// Serialized OCR results (JSON array of shape-text pairs)
        results_json: String,
    },

    /// A tool was selected
    ToolSelected {
        /// Name of the selected tool
        tool_name: String,
    },

    /// Custom event with arbitrary data
    Custom {
        /// Plugin that sent the event
        plugin: String,
        /// Event type identifier
        event_type: String,
        /// JSON-encoded event data
        data: String,
    },
}

impl AppEvent {
    /// Creates a custom event with JSON-serializable data.
    ///
    /// # Arguments
    /// * `plugin` - Name of the plugin sending the event
    /// * `event_type` - Type identifier for the event
    /// * `data` - Data to include (will be JSON-encoded)
    ///
    /// # Errors
    /// Returns an error if the data cannot be serialized to JSON.
    pub fn custom<T: serde::Serialize>(
        plugin: impl Into<String>,
        event_type: impl Into<String>,
        data: &T,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self::Custom {
            plugin: plugin.into(),
            event_type: event_type.into(),
            data: serde_json::to_string(data)?,
        })
    }

    /// Attempts to deserialize the data from a custom event.
    ///
    /// # Errors
    /// Returns an error if this is not a custom event or if deserialization fails.
    pub fn decode_custom<T: serde::de::DeserializeOwned>(&self) -> Result<T, DecodeError> {
        match self {
            Self::Custom { data, .. } => serde_json::from_str(data).map_err(DecodeError::Json),
            _ => Err(DecodeError::NotCustomEvent),
        }
    }
}

/// Error that can occur when decoding custom event data.
#[derive(Debug)]
pub enum DecodeError {
    /// The event is not a custom event
    NotCustomEvent,
    /// JSON deserialization failed
    Json(serde_json::Error),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::NotCustomEvent => write!(f, "Event is not a custom event"),
            DecodeError::Json(e) => write!(f, "JSON deserialization failed: {}", e),
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DecodeError::NotCustomEvent => None,
            DecodeError::Json(e) => Some(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_event_roundtrip() {
        #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
        struct TestData {
            value: i32,
            name: String,
        }

        let data = TestData {
            value: 42,
            name: "test".to_string(),
        };

        let event = AppEvent::custom("test_plugin", "test_event", &data).unwrap();
        let decoded: TestData = event.decode_custom().unwrap();

        assert_eq!(data, decoded);
    }
}
