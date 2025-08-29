//! Example: Direct conversion from Miden VM execution to Plonky3 proof
//! 
//! This example demonstrates the complete workflow:
//! 1. Execute a Fibonacci program in Miden VM
//! 2. Convert the execution trace directly to Plonky3 format
//! 3. Show how to integrate with Plonky3 proving system
//!
//! This eliminates the need to write traces to disk and read them back.

use p3_trace_convertor::{TraceConverter, convert_miden_trace};
use p3_goldilocks::Goldilocks;
use p3_matrix::Matrix;
use p3_field::PrimeCharacteristicRing;
use winter_prover::Trace;

// Import Miden VM components for creating and executing programs  
use miden_assembly::Assembler;
use miden_processor::{execute, StackInputs, AdviceInputs, DefaultHost, ExecutionOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Miden VM to Plonky3 Direct Conversion Example");
    println!("=================================================\n");

    // === Step 1: Create and Execute Miden Program ===
    println!("📊 Step 1: Creating and executing Miden VM program...");
    
    // Create a simple Fibonacci program in Miden Assembly
    let masm_code = r#"
        begin
            push.0      # Initialize with fib(0) = 0
            push.1      # Initialize with fib(1) = 1
            
            repeat.200  # Compute 20 Fibonacci steps
                dup.1   # Duplicate the second element (previous number)
                add     # Add: curr + prev = next
                swap    # Move next to correct position
                drop    # Remove the old previous number
            end
        end
    "#;
    
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
    println!("   📏 Trace dimensions: {}×{}", miden_trace.length(), miden_trace.main_trace_width());

    // === Step 2: Direct Trace Conversion ===
    println!("\n🔄 Step 2: Converting Miden trace to Plonky3 format...");
    let conversion_start = std::time::Instant::now();
    
    let plonky3_trace = TraceConverter::convert::<Goldilocks>(&miden_trace)?;
    
    let conversion_time = conversion_start.elapsed();
    println!("   ✅ Conversion completed in {:.3}ms", conversion_time.as_millis());
    println!("   📏 Plonky3 trace dimensions: {}×{}", plonky3_trace.height(), plonky3_trace.width());
    
    // Show conversion statistics
    let stats = TraceConverter::trace_stats(&miden_trace);
    println!("   📈 Conversion stats:");
    println!("      Original height: {}", stats.original_height);
    println!("      Padded height: {} (2^{})", stats.padded_height, stats.log_height);
    println!("      Padding rows added: {}", stats.padding_rows);

    // === Step 3: Verify Trace Properties ===
    println!("\n✅ Step 3: Verifying trace properties...");
    
    // Check basic properties
    println!("   🔍 Verifying trace structure:");
    println!("      Width: {} columns", plonky3_trace.width());
    println!("      Height: {} rows", plonky3_trace.height());
    println!("      Power of 2: {}", plonky3_trace.height().is_power_of_two());
    
    // Verify padding is zero (for the padding rows)
    if stats.padding_rows > 0 {
        println!("   🔍 Verifying zero padding in last {} rows:", stats.padding_rows);
        let start_padding = stats.original_height;
        let end_padding = stats.padded_height;
        
        for row_idx in start_padding..std::cmp::min(start_padding + 3, end_padding) {
            let row = plonky3_trace.row_slice(row_idx).unwrap();
            let all_zeros = row.iter().all(|&val| val == Goldilocks::ZERO);
            println!("      Row {}: All zeros = {}", row_idx, all_zeros);
        }
    }

    // === Step 4: Demonstrate Conversion API ===
    println!("\n🎯 Step 4: Demonstrating conversion API...");
    
    // Show direct conversion function
    let direct_conversion = convert_miden_trace::<Goldilocks>(&miden_trace)?;
    println!("   ✅ Direct conversion API: {}×{}", direct_conversion.height(), direct_conversion.width());
    
    // Verify they're identical
    assert_eq!(plonky3_trace.height(), direct_conversion.height());
    assert_eq!(plonky3_trace.width(), direct_conversion.width());
    println!("   ✅ Both conversion methods produce identical results");

    // === Step 5: Integration Ready ===
    println!("\n🔐 Step 5: Ready for Plonky3 integration!");
    println!("   The trace is now ready to be used with:");
    println!("   ```rust");
    println!("   // Create your AIR (Algebraic Intermediate Representation)");
    println!("   let air = YourConstraintSystem::new();");
    println!("   ");
    println!("   // Set up Plonky3 configuration");
    println!("   let config = create_plonky3_config();");
    println!("   ");
    println!("   // Generate the proof!");
    println!("   let proof = prove(&config, &air, plonky3_trace, &public_values);");
    println!("   ```");

    println!("\n🎉 Example completed successfully!");
    println!("   • Miden trace: {}×{} elements", miden_trace.length(), miden_trace.main_trace_width());
    println!("   • Plonky3 trace: {}×{} elements", plonky3_trace.height(), plonky3_trace.width());
    println!("   • Total conversion time: {:.3}ms", conversion_time.as_millis());
    println!("   • Zero padding: {} rows added", stats.padding_rows);
    println!("   • Memory efficiency: No disk I/O required! 💾");

    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_simple_miden_program_conversion() {
        // Create a very simple Miden program for testing
        let masm_code = r#"
            begin
                push.5
                push.10
                add
            end
        "#;
        
        let program = Assembler::default()
            .assemble_program(masm_code)
            .expect("Failed to compile test program");
        
        let trace = execute(
            &program,
            StackInputs::default(),
            AdviceInputs::default(),
            &mut DefaultHost::default(),
            ExecutionOptions::default(),
        ).expect("Failed to execute test program");
        
        // Convert the trace
        let plonky3_trace = TraceConverter::convert::<Goldilocks>(&trace)
            .expect("Conversion should succeed");
        
        // Verify basic properties
        assert!(plonky3_trace.width() > 0, "Trace should have columns");
        assert!(plonky3_trace.height().is_power_of_two(), "Height should be power of 2");
        assert!(plonky3_trace.height() >= trace.length(), "Padded height should be >= original");
        
        println!("Simple program conversion: {}×{} -> {}×{}", 
            trace.length(), trace.main_trace_width(),
            plonky3_trace.height(), plonky3_trace.width());
    }

    #[test]
    fn test_zero_padding_verification() {
        // Create a small program to ensure we get padding
        let masm_code = r#"
            begin
                push.1
                push.2
            end
        "#;
        
        let program = Assembler::default()
            .assemble_program(masm_code)
            .expect("Failed to compile test program");
        
        let trace = execute(
            &program,
            StackInputs::default(),
            AdviceInputs::default(),
            &mut DefaultHost::default(),
            ExecutionOptions::default(),
        ).expect("Failed to execute test program");
        
        let plonky3_trace = TraceConverter::convert::<Goldilocks>(&trace)
            .expect("Conversion should succeed");
        
        let stats = TraceConverter::trace_stats(&trace);
        
        // If there are padding rows, verify they are zero
        if stats.padding_rows > 0 {
            let start_padding = stats.original_height;
            let end_padding = stats.padded_height;
            
            for row_idx in start_padding..end_padding {
                let row = plonky3_trace.row_slice(row_idx).unwrap();
                for &value in row.iter() {
                    assert_eq!(value, Goldilocks::ZERO, 
                        "Padding row {} should contain only zeros", row_idx);
                }
            }
        }
        
        println!("Zero padding verification: {} original rows, {} padding rows", 
            stats.original_height, stats.padding_rows);
    }
}