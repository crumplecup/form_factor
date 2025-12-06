//! Performance measurement utilities for integration tests.

use std::time::Instant;

/// Result of a performance benchmark.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Name of the operation measured.
    pub operation: String,
    /// Duration in milliseconds.
    pub duration_ms: f64,
}

/// Measure the execution time of an operation.
///
/// Returns a `BenchmarkResult` containing the operation name and duration.
///
/// # Examples
///
/// ```
/// use botticelli_health::measure_operation;
///
/// let result = measure_operation("test_op", || {
///     // Some operation
///     std::thread::sleep(std::time::Duration::from_millis(10));
/// });
///
/// assert!(result.duration_ms >= 10.0);
/// ```
pub fn measure_operation<F>(operation: &str, f: F) -> BenchmarkResult
where
    F: FnOnce(),
{
    let start = Instant::now();
    f();
    let duration = start.elapsed();

    BenchmarkResult {
        operation: operation.to_string(),
        duration_ms: duration.as_secs_f64() * 1000.0,
    }
}
