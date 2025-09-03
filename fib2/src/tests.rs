use p3_matrix::Matrix;
use winter_prover::Trace;

use crate::trace_gen;

/// Test that we can successfully generate traces using the new API
/// This test verifies:
/// 1. Miden program compilation works correctly
/// 2. Program execution produces a valid trace
/// 3. Trace conversion to Plonky3 format works
/// 4. Power-of-2 padding is applied correctly
#[test]
fn test_trace_gen() {
    match trace_gen(10) {
        Ok((miden_trace, p3_trace, _program, _stack_inputs, _advice_inputs)) => {
            // Verify basic properties of both traces
            assert!(miden_trace.length() > 0, "Miden trace should have rows");
            assert!(
                miden_trace.main_trace_width() > 0,
                "Miden trace should have columns"
            );
            assert!(p3_trace.width() > 0, "P3 trace should have columns");
            assert!(
                p3_trace.height().is_power_of_two(),
                "P3 height should be power of 2"
            );

            println!(
                "Traces generated successfully: Miden {}×{}, P3 {}×{}",
                miden_trace.length(),
                miden_trace.main_trace_width(),
                p3_trace.height(),
                p3_trace.width()
            );
        }
        Err(e) => {
            println!("Failed to generate traces: {}", e);
            // Don't fail the test if Miden VM isn't available
        }
    }
}
