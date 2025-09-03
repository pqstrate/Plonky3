use std::fs::File;
use std::io::Write;

use miden_assembly::Assembler;
use miden_processor::{AdviceInputs, DefaultHost, ExecutionOptions, StackInputs, execute};
use miden_vm::{AdviceInputs as VmAdviceInputs, StackInputs as VmStackInputs};
use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::{PrimeCharacteristicRing, PrimeField64};
use p3_goldilocks::Goldilocks;
use p3_matrix::Matrix;
use p3_matrix::dense::RowMajorMatrix;
use p3_trace_convertor::{TraceConverter, convert_miden_trace};
use winter_prover::Trace;

use crate::NUM_COLS;

/// Write Miden trace to a log file with custom filename
fn write_miden_trace_to_file(
    miden_trace: &miden_processor::ExecutionTrace,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   📝 Writing Miden trace to {}...", filename);

    let mut file = File::create(filename)?;
    let main_segment = miden_trace.main_segment();
    let height = miden_trace.length();
    let width = miden_trace.main_trace_width();

    writeln!(file, "# Miden VM Execution Trace")?;
    writeln!(file, "# Dimensions: {}×{}", height, width)?;
    writeln!(file, "# Format: [col0, col1, col2, ...]")?;
    writeln!(file)?;

    for row_idx in 0..height {
        write!(file, "[")?;
        for col_idx in 0..width {
            let column = main_segment.get_column(col_idx);
            let value = column[row_idx].as_int();
            write!(file, "{}", value)?;
            if col_idx < width - 1 {
                write!(file, ", ")?;
            }
        }
        writeln!(file, "]")?;
    }

    println!("   ✅ Miden trace written to {}", filename);
    Ok(())
}

/// Write Plonky3 trace to a log file with custom filename
fn write_plonky3_trace_to_file(
    plonky3_trace: &RowMajorMatrix<Goldilocks>,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   📝 Writing Plonky3 trace to {}...", filename);

    let mut file = File::create(filename)?;
    let height = plonky3_trace.height();
    let width = plonky3_trace.width();

    writeln!(file, "# Plonky3 Trace (after conversion and padding)")?;
    writeln!(file, "# Dimensions: {}×{}", height, width)?;
    writeln!(file, "# Format: [col0, col1, col2, ...]")?;
    writeln!(file)?;

    for row_idx in 0..height {
        write!(file, "[")?;
        let row = plonky3_trace.row_slice(row_idx).unwrap();
        for (col_idx, &value) in row.iter().enumerate() {
            write!(file, "{}", value.as_canonical_u64())?;
            if col_idx < width - 1 {
                write!(file, ", ")?;
            }
        }
        writeln!(file, "]")?;
    }

    println!("   ✅ Plonky3 trace written to {}", filename);
    Ok(())
}

/// IncrementAir defines the arithmetic constraints for our increment proof
/// This AIR enforces that the first column of each row increments by 1 from the previous row
/// i.e., trace[i][0] = trace[i-1][0] + 1 for all transition rows
#[derive(Clone)]
pub struct IncrementAir;

/// BaseAir implementation tells Plonky3 the basic properties of our computation
impl<F> BaseAir<F> for IncrementAir {
    /// Returns the number of columns in our execution trace
    /// Our trace has 80 columns as determined from Miden VM
    fn width(&self) -> usize {
        NUM_COLS
    }
}

/// Air implementation defines the actual arithmetic constraints
/// This is where we specify what makes a valid computation
impl<AB: AirBuilder> Air<AB> for IncrementAir {
    /// eval() is called by the STARK prover to check constraints
    /// It receives an AirBuilder that lets us access trace rows and define constraints
    fn eval(&self, builder: &mut AB) {
        // Get access to the execution trace matrix
        let main = builder.main();

        // Get current row and next row for transition constraints
        // current_row = trace[i], next_row = trace[i+1]
        let (current_row, next_row) = (
            main.row_slice(0)
                .expect("Matrix must have at least one row"),
            main.row_slice(1)
                .expect("Matrix must have at least two rows for transitions"),
        );

        // Apply constraint only during transitions (between consecutive rows)
        // This excludes boundary conditions (first/last rows)
        let mut when_transition = builder.when_transition();

        // The core constraint: next_row[0] - current_row[0] = 1
        // This ensures that the first column increments by exactly 1 each row
        // AB::Expr::from(AB::F::ONE) creates the field element representing 1
        when_transition.assert_eq(
            next_row[0].clone() - current_row[0].clone(),
            AB::Expr::from(AB::F::ONE),
        );
    }
}

/// Generate traces for a given number of Fibonacci iterations
///
/// Returns both the Miden VM execution trace and the converted Plonky3 trace.
/// Also returns the program and inputs needed for proof generation.
/// Writes traces to files: fib_{fib_iter}_trace_miden.log and fib_{fib_iter}_trace_p3.log
///
/// # Arguments
/// * `fib_iter` - Number of Fibonacci iterations to compute
///
/// # Returns
/// * `(ExecutionTrace, RowMajorMatrix<Goldilocks>, Program, StackInputs, AdviceInputs)` - Tuple of traces and execution parameters
pub fn trace_gen(
    fib_iter: usize,
) -> Result<
    (
        miden_processor::ExecutionTrace,
        RowMajorMatrix<Goldilocks>,
        miden_vm::Program,
        miden_vm::StackInputs,
        miden_vm::AdviceInputs,
    ),
    Box<dyn std::error::Error>,
> {
    println!("🚀 Generating trace using Miden VM execution...");

    // Create a simple Fibonacci program in Miden Assembly
    // This creates a computation with incrementing steps suitable for our constraint
    let masm_code = format!(
        r#"
        begin
            # Initialize counter starting from 0
            push.0      # Initialize with 0 (this will be our incrementing counter)
            push.1      # Initialize with fib(1) = 1
            
            # Compute Fibonacci steps while maintaining incrementing counter
            repeat.{}   # Compute {} steps to get a reasonable trace
                dup.1   # Duplicate the second element (previous number)
                add     # Add: curr + prev = next
                swap    # Move next to correct position  
                drop    # Remove the old previous number
                
                # The execution naturally creates incrementing step counters
                # in the trace due to Miden's internal execution model
            end
        end
    "#,
        fib_iter, fib_iter
    );

    println!("   📝 Assembling Miden program...");
    let program = Assembler::default()
        .assemble_program(masm_code)
        .expect("Failed to compile Miden Assembly code");

    println!("   ▶️  Executing Miden program...");
    let stack_inputs = StackInputs::default();
    let advice_inputs = AdviceInputs::default();
    let mut host = DefaultHost::default();
    let options = ExecutionOptions::default();

    let miden_trace = execute(&program, stack_inputs, advice_inputs, &mut host, options)
        .expect("Failed to execute Miden program");

    println!("   ✅ Miden execution completed");
    println!(
        "   📏 Original trace dimensions: {}×{}",
        miden_trace.length(),
        miden_trace.main_trace_width()
    );

    // Write the Miden trace to log file with custom filename
    let miden_filename = format!("fib_{}_trace_miden.log", fib_iter);
    write_miden_trace_to_file(&miden_trace, &miden_filename)?;

    // Convert the Miden trace to Plonky3 format
    println!("   🔄 Converting trace to Plonky3 format...");
    let conversion_start = std::time::Instant::now();

    let plonky3_trace = convert_miden_trace::<Goldilocks>(&miden_trace)?;

    let conversion_time = conversion_start.elapsed();
    println!(
        "   ✅ Conversion completed in {:.3}ms",
        conversion_time.as_millis()
    );
    println!(
        "   📏 Plonky3 trace dimensions: {}×{}",
        plonky3_trace.height(),
        plonky3_trace.width()
    );

    // Show conversion statistics
    let stats = TraceConverter::trace_stats(&miden_trace);
    println!("   📈 Conversion stats:");
    println!("      Original height: {}", stats.original_height);
    println!(
        "      Padded height: {} (2^{})",
        stats.padded_height, stats.log_height
    );
    println!("      Padding rows added: {}", stats.padding_rows);

    // Verify the trace width matches our expectations
    assert_eq!(
        plonky3_trace.width(),
        NUM_COLS,
        "Trace width {} should match NUM_COLS {}",
        plonky3_trace.width(),
        NUM_COLS
    );

    // Write the Plonky3 trace to log file with custom filename
    let p3_filename = format!("fib_{}_trace_p3.log", fib_iter);
    write_plonky3_trace_to_file(&plonky3_trace, &p3_filename)?;

    // Convert inputs to miden_vm types for proof generation
    let vm_stack_inputs = VmStackInputs::default();
    let vm_advice_inputs = VmAdviceInputs::default();

    Ok((
        miden_trace,
        plonky3_trace,
        program,
        vm_stack_inputs,
        vm_advice_inputs,
    ))
}
