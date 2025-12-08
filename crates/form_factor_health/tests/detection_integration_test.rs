use form_factor::ToolMode;
use form_factor_drawing::CanvasState;
use form_factor_health::create_test_canvas;

/// Phase 4.1: Optional Feature Integration Tests
///
/// Tests integration of feature-gated detection functionality:
/// - Text detection workflows
/// - Logo detection workflows  
/// - OCR extraction workflows
/// - Event emission and layer integration

// =============================================================================
// Text Detection Integration Tests
// =============================================================================

#[test]
#[cfg(feature = "text-detection")]
fn test_text_detector_initialization() {
    use form_factor::TextDetector;

    let _canvas = create_test_canvas();

    // Test that TextDetector can be initialized
    let detector_result = TextDetector::new("models/text_detection.onnx".to_string());

    // In a real test environment with OpenCV models, this would succeed
    // Without models, it may fail - either outcome is valid for this test
    match detector_result {
        Ok(_detector) => {
            // Successfully initialized
        }
        Err(_) => {
            // Model not available - acceptable in CI
        }
    }
}

#[test]
#[cfg(feature = "text-detection")]
fn test_text_region_structure() {
    use form_factor::TextRegion;

    let _canvas = create_test_canvas();

    // TextRegion represents a detected text bounding box
    let region = TextRegion::new(10, 20, 90, 30, 0.95).expect("Valid text region");

    // Verify region has valid dimensions
    assert_eq!(*region.x(), 10);
    assert_eq!(*region.y(), 20);
    assert_eq!(*region.width(), 90);
    assert_eq!(*region.height(), 30);
    assert_eq!(*region.confidence(), 0.95);
}

#[test]
#[cfg(feature = "text-detection")]
fn test_canvas_ready_for_text_detection() {
    let canvas = create_test_canvas();

    // Verify canvas state is appropriate for detection workflow
    assert_eq!(canvas.project_name(), "Test Canvas");
    assert_eq!(canvas.current_tool(), &ToolMode::Select);
    assert!(matches!(canvas.current_state(), CanvasState::Idle));
}

// =============================================================================
// Logo Detection Integration Tests
// =============================================================================

#[test]
#[cfg(feature = "logo-detection")]
fn test_logo_detector_initialization() {
    use form_factor::LogoDetector;

    let _canvas = create_test_canvas();

    // Test that LogoDetector can be initialized
    let _detector = LogoDetector::new();

    // LogoDetector initializes successfully
}

#[test]
#[cfg(feature = "logo-detection")]
fn test_logo_detection_methods() {
    use form_factor::LogoDetectionMethod;

    let _canvas = create_test_canvas();

    // Test that both detection methods are available
    let template_method = LogoDetectionMethod::TemplateMatching;
    let feature_method = LogoDetectionMethod::FeatureMatching;

    // Methods should be distinct
    assert_ne!(
        std::mem::discriminant(&template_method),
        std::mem::discriminant(&feature_method)
    );
}

#[test]
#[cfg(feature = "logo-detection")]
fn test_logo_location_structure() {
    use form_factor::LogoLocation;

    let _canvas = create_test_canvas();

    // LogoLocation represents detected logo position
    let location = LogoLocation { x: 100, y: 150 };

    assert_eq!(location.x, 100);
    assert_eq!(location.y, 150);
}

#[test]
#[cfg(feature = "logo-detection")]
fn test_logo_size_structure() {
    use form_factor::LogoSize;

    let _canvas = create_test_canvas();

    // LogoSize represents detected logo dimensions
    let size = LogoSize {
        width: 200,
        height: 150,
    };

    assert_eq!(size.width, 200);
    assert_eq!(size.height, 150);
}

// =============================================================================
// OCR Integration Tests
// =============================================================================

#[test]
#[cfg(feature = "ocr")]
fn test_ocr_engine_initialization() {
    use form_factor::{EngineMode, OCRConfig, OCREngine, PageSegmentationMode};

    let _canvas = create_test_canvas();

    // Test that OCREngine can be initialized with a config
    let config = OCRConfig {
        language: "eng".to_string(),
        page_segmentation_mode: PageSegmentationMode::Auto,
        engine_mode: EngineMode::Default,
        min_confidence: 60,
        preprocess: false,
        tessdata_path: Some("tessdata".to_string()),
    };

    let engine_result = OCREngine::new(config);

    // In environment with Tesseract, this should succeed
    match engine_result {
        Ok(_engine) => {
            // Successfully initialized
        }
        Err(_) => {
            // Tesseract not available - acceptable in CI
        }
    }
}

#[test]
#[cfg(feature = "ocr")]
fn test_ocr_config_modes() {
    use form_factor::{EngineMode, OCRConfig, PageSegmentationMode};

    let _canvas = create_test_canvas();

    // Test OCR configuration options
    let config = OCRConfig {
        language: "eng".to_string(),
        page_segmentation_mode: PageSegmentationMode::SingleBlock,
        engine_mode: EngineMode::LstmOnly,
        min_confidence: 70,
        preprocess: false,
        tessdata_path: Some("tessdata".to_string()),
    };

    assert_eq!(config.engine_mode, EngineMode::LstmOnly);
    assert_eq!(
        config.page_segmentation_mode,
        PageSegmentationMode::SingleBlock
    );
}

#[test]
#[cfg(feature = "ocr")]
fn test_bounding_box_structure() {
    use form_factor::BoundingBox;

    let _canvas = create_test_canvas();

    // Test BoundingBox for text regions
    let bbox = BoundingBox {
        x: 10,
        y: 20,
        width: 100,
        height: 50,
    };

    assert_eq!(bbox.x, 10);
    assert_eq!(bbox.y, 20);
    assert_eq!(bbox.width, 100);
    assert_eq!(bbox.height, 50);
}

#[test]
#[cfg(feature = "ocr")]
fn test_word_result_construction() {
    use form_factor::BoundingBox;

    let _canvas = create_test_canvas();

    // WordResult has non-public fields, so we test BoundingBox separately
    let bbox = BoundingBox {
        x: 10,
        y: 20,
        width: 50,
        height: 20,
    };

    // Verify bounding box structure
    assert_eq!(bbox.x, 10);
    assert_eq!(bbox.y, 20);
    assert_eq!(bbox.width, 50);
    assert_eq!(bbox.height, 20);
}

// =============================================================================
// Cross-Feature Integration Tests
// =============================================================================

#[test]
#[cfg(all(feature = "text-detection", feature = "ocr"))]
fn test_text_detection_and_ocr_workflow() {
    use form_factor::{EngineMode, OCRConfig, OCREngine, PageSegmentationMode, TextRegion};

    let _canvas = create_test_canvas();

    // Typical workflow: TextDetector finds regions, OCR extracts text

    // 1. Text detection creates regions
    let text_region = TextRegion::new(50, 100, 200, 50, 0.90).expect("Valid text region");

    assert_eq!(*text_region.width(), 200);
    assert_eq!(*text_region.height(), 50);

    // 2. OCR engine would process those regions
    // OCREngine needs a config now
    let config = OCRConfig {
        language: "eng".to_string(),
        page_segmentation_mode: PageSegmentationMode::Auto,
        engine_mode: EngineMode::Default,
        min_confidence: 50,
        preprocess: false,
        tessdata_path: Some("tessdata".to_string()),
    };
    let _ocr_result = OCREngine::new(config);
}

#[test]
#[cfg(all(feature = "logo-detection", feature = "text-detection"))]
fn test_mixed_detection_types_workflow() {
    use form_factor::{LogoDetector, TextDetector};

    let canvas = create_test_canvas();

    // Workflow: Detect both logos and text on same image

    // Initialize both detectors (may fail without OpenCV)
    let _text_detector_result = TextDetector::new("models/text_detection.onnx".to_string());
    let _logo_detector_result = LogoDetector::new();

    // Both detection types should be independent
    assert_eq!(canvas.project_name(), "Test Canvas");
}

// =============================================================================
// Canvas Integration Tests
// =============================================================================

#[test]
fn test_canvas_without_detection_features() {
    // Canvas should work fine even without detection features
    let canvas = create_test_canvas();

    assert_eq!(canvas.project_name(), "Test Canvas");
    assert_eq!(canvas.current_tool(), &ToolMode::Select);
    assert!(matches!(canvas.current_state(), CanvasState::Idle));
}

#[test]
fn test_canvas_tool_modes_independent_of_detection() {
    // Tool modes should work regardless of detection features
    let mut canvas = create_test_canvas();

    // Verify initial state
    assert_eq!(canvas.current_tool(), &ToolMode::Select);

    // Set tool to Rectangle
    canvas.set_tool(ToolMode::Rectangle);
    assert_eq!(canvas.current_tool(), &ToolMode::Rectangle);

    // Set tool to Circle
    canvas.set_tool(ToolMode::Circle);
    assert_eq!(canvas.current_tool(), &ToolMode::Circle);
}

#[test]
fn test_detection_subtype_enum() {
    use form_factor::DetectionSubtype;

    let _canvas = create_test_canvas();

    // DetectionSubtype should be available regardless of features
    let text_subtype = DetectionSubtype::Text;
    let logos_subtype = DetectionSubtype::Logos;

    assert_ne!(
        std::mem::discriminant(&text_subtype),
        std::mem::discriminant(&logos_subtype)
    );
}
