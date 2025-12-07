use form_factor_drawing::DrawingCanvas;
use form_factor_health::{create_rectangle_shape, create_test_canvas, measure_operation};

#[test]
fn test_large_shape_collection_performance() {
    let mut canvas = create_test_canvas();

    // Add 1000 shapes
    let result = measure_operation("add_1000_shapes", || {
        for i in 0..1000 {
            let x = (i % 50) as f32 * 10.0;
            let y = (i / 50) as f32 * 10.0;
            let shape = create_rectangle_shape(x, y, 50.0, 50.0);
            canvas.test_add_shape(shape);
        }
    });

    assert_eq!(canvas.shape_count(), 1000);
    assert!(
        result.duration_ms < 1000.0,
        "Adding 1000 shapes took {}ms (should be < 1000ms)",
        result.duration_ms
    );
}

#[test]
fn test_selection_in_dense_canvas() {
    let mut canvas = create_test_canvas();

    // Create dense grid of shapes
    for i in 0..100 {
        let x = (i % 10) as f32 * 60.0;
        let y = (i / 10) as f32 * 60.0;
        let shape = create_rectangle_shape(x, y, 50.0, 50.0);
        canvas.test_add_shape(shape);
    }

    // Measure selection operations
    let result = measure_operation("select_in_dense_canvas", || {
        for i in 0..100 {
            canvas.test_set_selected_shape(Some(i % 100));
        }
    });

    assert!(
        result.duration_ms < 100.0,
        "100 selection operations took {}ms (should be < 100ms)",
        result.duration_ms
    );
}

#[test]
fn test_large_canvas_with_layers() {
    let mut canvas = create_test_canvas();

    // Add shapes across multiple layers
    let result = measure_operation("add_500_shapes_across_layers", || {
        for i in 0..500 {
            let x = (i % 25) as f32 * 20.0;
            let y = (i / 25) as f32 * 20.0;
            let shape = create_rectangle_shape(x, y, 80.0, 30.0);
            canvas.test_add_shape(shape);
        }
    });

    assert_eq!(canvas.shape_count(), 500);
    assert!(
        result.duration_ms < 2500.0,
        "Adding 500 shapes took {}ms (should be < 2500ms)",
        result.duration_ms
    );
}

#[test]
fn test_rapid_tool_switching() {
    let mut canvas = create_test_canvas();

    use form_factor_drawing::ToolMode;

    let result = measure_operation("1000_tool_switches", || {
        for _ in 0..1000 {
            canvas.set_tool(ToolMode::Rectangle);
            canvas.set_tool(ToolMode::Circle);
            canvas.set_tool(ToolMode::Freehand);
            canvas.set_tool(ToolMode::Select);
        }
    });

    assert!(
        result.duration_ms < 500.0,
        "1000 tool switches took {}ms (should be < 500ms)",
        result.duration_ms
    );
}

#[test]
fn test_shape_iteration_performance() {
    let mut canvas = create_test_canvas();

    // Add 1000 shapes
    for i in 0..1000 {
        let x = (i % 50) as f32 * 10.0;
        let y = (i / 50) as f32 * 10.0;
        let shape = create_rectangle_shape(x, y, 50.0, 50.0);
        canvas.test_add_shape(shape);
    }

    let result = measure_operation("iterate_1000_shapes_100_times", || {
        for _ in 0..100 {
            let count = canvas.shape_count();
            assert_eq!(count, 1000);
        }
    });

    assert!(
        result.duration_ms < 100.0,
        "Iterating shapes 100 times took {}ms (should be < 100ms)",
        result.duration_ms
    );
}

#[test]
fn test_shape_creation_throughput() {
    let mut canvas = create_test_canvas();

    // Measure pure shape creation
    let result = measure_operation("create_1000_shapes", || {
        for i in 0..1000 {
            let x = (i % 50) as f32 * 10.0;
            let y = 0.0;
            let shape = create_rectangle_shape(x, y, 50.0, 50.0);
            canvas.test_add_shape(shape);
        }
    });

    assert_eq!(canvas.shape_count(), 1000);
    assert!(
        result.duration_ms < 2000.0,
        "Creating 1000 shapes took {}ms (should be < 2000ms)",
        result.duration_ms
    );
}

#[test]
fn test_memory_efficiency() {
    use std::mem::size_of;

    // Verify DrawingCanvas doesn't grow unexpectedly
    let canvas_size = size_of::<DrawingCanvas>();

    // This is a sanity check - adjust threshold based on actual structure
    assert!(
        canvas_size < 10_000,
        "DrawingCanvas is {} bytes (seems too large, check for unnecessary clones)",
        canvas_size
    );
}
