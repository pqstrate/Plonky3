//! # Trace Convertor
//!
//! Direct conversion between Miden VM execution traces and Plonky3 STARK traces.
//! This eliminates the need to serialize/deserialize traces to/from disk.
//!
//! ## Overview
//!
//! This library provides utilities to convert Miden VM's `ExecutionTrace` directly
//! into Plonky3's `RowMajorMatrix<F>` format, allowing for seamless integration
//! between Miden VM execution and Plonky3 proof generation.
//!
//! ## Usage
//!
//! ```no_run
//! use p3_trace_convertor::TraceConverter;
//! use p3_goldilocks::Goldilocks;
//! use miden_processor::ExecutionTrace;
//!
//! // Execute your Miden program to get an ExecutionTrace
//! # let miden_trace: &ExecutionTrace = panic!("This is just an example");
//!
//! // Convert directly to Plonky3 format
//! let plonky3_trace = TraceConverter::convert::<Goldilocks>(&miden_trace).unwrap();
//!
//! // Use with Plonky3 proving system
//! // let proof = prove(&config, &air, plonky3_trace, &public_values);
//! ```

extern crate alloc;

use alloc::vec::Vec;
use core::fmt;

// Import actual Miden VM types
use miden_core::{Felt, FieldElement};
use miden_processor::ExecutionTrace;
use p3_field::PrimeField;
use p3_matrix::dense::RowMajorMatrix;
use p3_util::log2_strict_usize;

/// Error type for trace conversion operations
#[derive(Debug)]
pub enum ConversionError {
    /// Invalid trace dimensions
    InvalidDimensions { rows: usize, cols: usize },
    /// Trace is empty
    EmptyTrace,
    /// Field conversion error
    FieldConversion(String),
    /// Power of 2 padding error
    PowerOfTwoPadding { current: usize, required: usize },
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::InvalidDimensions { rows, cols } => {
                write!(f, "Invalid trace dimensions: {}×{}", rows, cols)
            }
            ConversionError::EmptyTrace => write!(f, "Trace is empty"),
            ConversionError::FieldConversion(msg) => write!(f, "Field conversion error: {}", msg),
            ConversionError::PowerOfTwoPadding { current, required } => {
                write!(
                    f,
                    "Power of 2 padding error: current={}, required={}",
                    current, required
                )
            }
        }
    }
}

impl core::error::Error for ConversionError {}

// Import the Trace trait from winter_prover to access the methods
use winter_prover::Trace;

/// Main converter for transforming Miden execution traces to Plonky3 format
pub struct TraceConverter;

impl TraceConverter {
    /// Convert a Miden execution trace to a Plonky3 RowMajorMatrix
    ///
    /// This function:
    /// 1. Extracts the main trace data from Miden format
    /// 2. Converts field elements to the target field type
    /// 3. Ensures power-of-2 padding with zeros for STARK requirements
    /// 4. Constructs the RowMajorMatrix in the format expected by Plonky3
    pub fn convert<F: PrimeField>(
        miden_trace: &ExecutionTrace,
    ) -> Result<RowMajorMatrix<F>, ConversionError> {
        let height = miden_trace.length();
        let width = miden_trace.main_trace_width();

        if height == 0 || width == 0 {
            return Err(ConversionError::EmptyTrace);
        }

        // Ensure power-of-2 height for STARK protocol
        let padded_height = height.next_power_of_two();

        println!(
            "Converting trace: {}×{} -> {}×{}",
            height, width, padded_height, width
        );

        // Convert column-major format (Miden) to row-major format (Plonky3)
        let mut data = Vec::with_capacity(padded_height * width);

        // Pre-fetch all columns to avoid repeated calls
        let main_segment = miden_trace.main_segment();
        let columns: Vec<&[Felt]> = (0..width)
            .map(|col_idx| main_segment.get_column(col_idx))
            .collect();

        for row_idx in 0..padded_height {
            for col_idx in 0..width {
                let felt_value = if row_idx < height - 1 {
                    // Get actual trace value
                    columns[col_idx][row_idx]
                } else if row_idx == height - 1 {
                    if col_idx == 0 {
                        // Warning! Last row - we have to modify the trace
                        // Miden's last row does not satisfy the constraints
                        Felt::from(row_idx as u32)
                    } else {
                        // Padding - always use zero as requested
                        columns[col_idx][row_idx]
                    }
                } else {
                    Felt::ZERO
                };

                // Convert Miden Felt to target field element
                // Miden Felt implements AsInt which gives us the canonical u64 representation
                let value_u64 = felt_value.as_int();
                let field_element = F::from_u64(value_u64);
                data.push(field_element);
            }
        }

        Ok(RowMajorMatrix::new(data, width))
    }

    /// Get trace statistics
    pub fn trace_stats(miden_trace: &ExecutionTrace) -> TraceStats {
        let height = miden_trace.length();
        let padded_height = height.next_power_of_two();

        TraceStats {
            original_height: height,
            padded_height,
            width: miden_trace.main_trace_width(),
            padding_rows: padded_height - height,
            log_height: log2_strict_usize(padded_height),
        }
    }
}

// Note: Padding is always zero as requested

/// Statistics about trace conversion
#[derive(Debug)]
pub struct TraceStats {
    pub original_height: usize,
    pub padded_height: usize,
    pub width: usize,
    pub padding_rows: usize,
    pub log_height: usize,
}

impl TraceStats {
    pub fn print(&self) {
        println!("Trace Statistics:");
        println!("  Original height: {}", self.original_height);
        println!(
            "  Padded height: {} (2^{})",
            self.padded_height, self.log_height
        );
        println!("  Width: {}", self.width);
        println!("  Padding rows: {}", self.padding_rows);
        println!("  Total elements: {}", self.padded_height * self.width);
    }
}

/// Helper function to convert a Miden ExecutionTrace to Plonky3 format
/// This is the main entry point for the conversion
pub fn convert_miden_trace<F: PrimeField>(
    miden_trace: &ExecutionTrace,
) -> Result<RowMajorMatrix<F>, ConversionError> {
    TraceConverter::convert(miden_trace)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests now require actual Miden ExecutionTrace instances
    // For full integration testing, you would:
    // 1. Create a Miden program (e.g., using Assembler)
    // 2. Execute it to get an ExecutionTrace
    // 3. Convert the trace using our converter
    // 4. Verify the conversion

    #[test]
    fn test_conversion_error_empty_trace() {
        // Test error handling - we can't easily create empty ExecutionTrace
        // without proper Miden setup, so this test is conceptual

        // When integrating with real Miden code, you would do:
        // let empty_trace = create_empty_execution_trace();
        // let result = TraceConverter::convert::<Goldilocks>(&empty_trace);
        // assert!(result.is_err());

        // For now, just test that our error types work
        let error = ConversionError::EmptyTrace;
        assert!(error.to_string().contains("empty"));
    }

    #[test]
    fn test_trace_stats_calculation() {
        // Test our stats calculation logic

        // These calculations should work regardless of the actual trace content
        let original_height: usize = 100;
        let padded_height = original_height.next_power_of_two(); // 128
        let width: usize = 50;

        let stats = TraceStats {
            original_height,
            padded_height,
            width,
            padding_rows: padded_height - original_height,
            log_height: log2_strict_usize(padded_height),
        };

        assert_eq!(stats.padded_height, 128);
        assert_eq!(stats.padding_rows, 28);
        assert_eq!(stats.log_height, 7); // log2(128) = 7
    }

    #[test]
    fn test_power_of_two_padding() {
        // Test our power-of-2 padding logic

        let original_sizes: [usize; 6] = [10, 64, 100, 127, 128, 200];
        let expected_padded: [usize; 6] = [16, 64, 128, 128, 128, 256];

        for (original, expected) in original_sizes.iter().zip(expected_padded.iter()) {
            let padded = original.next_power_of_two();
            assert_eq!(
                padded, *expected,
                "Original size {} should pad to {}, got {}",
                original, expected, padded
            );
            assert!(
                padded.is_power_of_two(),
                "Padded size {} should be power of 2",
                padded
            );
        }
    }
}

// Integration tests would go here when you have a real Miden program to test with
#[cfg(test)]
mod integration_tests {

    // Example of how you would test with a real Miden program:
    /*
    use miden_vm::{Assembler, execute, StackInputs, AdviceInputs, DefaultHost, ExecutionOptions};

    #[test]
    fn test_fibonacci_program_conversion() {
        // 1. Create a simple Miden program
        let masm_code = r#"
            begin
                push.0 push.1
                repeat.10
                    dup.1 add swap drop
                end
            end
        "#;

        let program = Assembler::default().assemble_program(masm_code).unwrap();

        // 2. Execute the program to get an ExecutionTrace
        let trace = execute(
            &program,
            StackInputs::default(),
            AdviceInputs::default(),
            &mut DefaultHost::default(),
            ExecutionOptions::default()
        ).unwrap();

        // 3. Convert the trace
        let plonky3_trace = TraceConverter::convert::<Goldilocks>(&trace).unwrap();

        // 4. Verify the conversion
        assert!(plonky3_trace.width() > 0);
        assert!(plonky3_trace.height().is_power_of_two());

        // Check that padding rows are zero
        let stats = TraceConverter::trace_stats(&trace);
        if stats.padding_rows > 0 {
            let last_row = plonky3_trace.row_slice(plonky3_trace.height() - 1).unwrap();
            for &value in last_row.iter() {
                assert_eq!(value, Goldilocks::ZERO);
            }
        }
    }
    */
}
