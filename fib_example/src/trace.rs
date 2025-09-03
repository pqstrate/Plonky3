//! # Fibonacci Execution Trace Generation
//! 
//! The execution trace is the step-by-step record of the computation.
//! For Fibonacci, this means recording each pair of consecutive Fibonacci numbers.

use p3_field::PrimeField64;
use p3_matrix::{dense::RowMajorMatrix, Matrix};
use core::borrow::Borrow;
use crate::{FibonacciRow, NUM_FIBONACCI_COLS};

/// Generate the execution trace for Fibonacci computation
/// 
/// ## Parameters:
/// - `a`: First Fibonacci number (usually 1)
/// - `b`: Second Fibonacci number (usually 1)  
/// - `num_steps`: Number of computation steps (must be power of 2)
/// 
/// ## Returns:
/// A matrix where each row contains (F(i), F(i+1)) for consecutive Fibonacci numbers
/// 
/// ## Example:
/// For `generate_fibonacci_trace(1, 1, 8)`:
/// ```
/// Row 0: (1, 1)   // F(1)=1, F(2)=1
/// Row 1: (1, 2)   // F(2)=1, F(3)=2  
/// Row 2: (2, 3)   // F(3)=2, F(4)=3
/// Row 3: (3, 5)   // F(4)=3, F(5)=5
/// Row 4: (5, 8)   // F(5)=5, F(6)=8
/// Row 5: (8, 13)  // F(6)=8, F(7)=13
/// Row 6: (13, 21) // F(7)=13, F(8)=21
/// Row 7: (21, 34) // F(8)=21, F(9)=34
/// ```
pub fn generate_fibonacci_trace<F: PrimeField64>(a: u64, b: u64, num_steps: usize) -> RowMajorMatrix<F> {
    // Ensure num_steps is a power of 2 (required by most PCS schemes)
    assert!(num_steps.is_power_of_two(), "Number of steps must be a power of 2");
    assert!(num_steps > 0, "Must have at least one step");
    
    // Create matrix to hold the trace
    let mut trace = RowMajorMatrix::new(
        F::zero_vec(num_steps * NUM_FIBONACCI_COLS), 
        NUM_FIBONACCI_COLS
    );
    
    // Get a mutable view of the trace as FibonacciRow structs
    let (prefix, rows, suffix) = unsafe { trace.values.align_to_mut::<FibonacciRow<F>>() };
    assert!(prefix.is_empty(), "Alignment should match");
    assert!(suffix.is_empty(), "Alignment should match");
    assert_eq!(rows.len(), num_steps);
    
    // Initialize first row with starting values
    rows[0] = FibonacciRow::new(F::from_u64(a), F::from_u64(b));
    
    // Generate subsequent rows using Fibonacci recurrence
    for i in 1..num_steps {
        // F(n+1) = F(n-1) + F(n)
        // new_left = old_right
        // new_right = old_left + old_right
        rows[i].left = rows[i - 1].right;
        rows[i].right = rows[i - 1].left + rows[i - 1].right;
    }
    
    trace
}

/// Generate trace and return both the trace and the final Fibonacci number
/// 
/// This is useful when you want to know what the result should be
/// without manually calculating it.
pub fn generate_fibonacci_trace_with_result<F: PrimeField64>(
    a: u64, 
    b: u64, 
    num_steps: usize
) -> (RowMajorMatrix<F>, u64) {
    let trace = generate_fibonacci_trace(a, b, num_steps);
    
    // Extract the final result from the last row
    let final_result = {
        let last_row = trace.row_slice(num_steps - 1).unwrap();
        let last_fib_row: &FibonacciRow<F> = (&*last_row).borrow();
        last_fib_row.right.as_canonical_u64()
    };
    
    (trace, final_result)
}

/// Calculate the nth Fibonacci number directly (for verification)
/// 
/// This is a simple iterative implementation for testing purposes.
/// In practice, you'd want to use the trace generation for the actual proof.
pub fn calculate_fibonacci(n: usize) -> u64 {
    if n == 0 {
        return 0;
    }
    if n <= 2 {
        return 1;
    }
    
    let mut a = 1u64;  // F(1)
    let mut b = 1u64;  // F(2)
    
    for _ in 3..=n {
        let next = a + b;
        a = b;
        b = next;
    }
    
    b
}

/// Print a trace in a human-readable format (for debugging/education)
pub fn print_trace<F: PrimeField64>(trace: &RowMajorMatrix<F>, title: &str) {
    println!("\n=== {} ===", title);
    println!("Trace dimensions: {} rows × {} columns", trace.height(), trace.width());
    
    for i in 0..trace.height() {
        let row = trace.row_slice(i).unwrap();
        let fib_row: &FibonacciRow<F> = (&*row).borrow();
        println!(
            "Row {:2}: F({:2}) = {:3}, F({:2}) = {:3}", 
            i, 
            i + 1, fib_row.left.as_canonical_u64(),
            i + 2, fib_row.right.as_canonical_u64()
        );
    }
    
    // Show the final result
    if trace.height() > 0 {
        let last_row = trace.row_slice(trace.height() - 1).unwrap();
        let last_fib_row: &FibonacciRow<F> = (&*last_row).borrow();
        println!("Final result: F({}) = {}", trace.height() + 1, last_fib_row.right.as_canonical_u64());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use p3_baby_bear::BabyBear;
    use p3_field::PrimeField32;
    use core::borrow::Borrow;
    
    #[test]
    fn test_fibonacci_calculation() {
        // Test the direct calculation function
        assert_eq!(calculate_fibonacci(1), 1);
        assert_eq!(calculate_fibonacci(2), 1);
        assert_eq!(calculate_fibonacci(3), 2);
        assert_eq!(calculate_fibonacci(4), 3);
        assert_eq!(calculate_fibonacci(5), 5);
        assert_eq!(calculate_fibonacci(8), 21);
        assert_eq!(calculate_fibonacci(10), 55);
    }
    
    #[test]
    fn test_trace_generation() {
        let trace = generate_fibonacci_trace::<BabyBear>(1, 1, 8);
        
        // Check dimensions
        assert_eq!(trace.height(), 8);
        assert_eq!(trace.width(), NUM_FIBONACCI_COLS);
        
        // Check first row
        let first_row = trace.row_slice(0).unwrap();
        let first_fib_row: &FibonacciRow<BabyBear> = (&*first_row).borrow();
        assert_eq!(first_fib_row.left.as_canonical_u32(), 1);
        assert_eq!(first_fib_row.right.as_canonical_u32(), 1);
        
        // Check a few more rows manually
        let second_row = trace.row_slice(1).unwrap();
        let second_fib_row: &FibonacciRow<BabyBear> = (&*second_row).borrow();
        assert_eq!(second_fib_row.left.as_canonical_u32(), 1);
        assert_eq!(second_fib_row.right.as_canonical_u32(), 2);
        
        let third_row = trace.row_slice(2).unwrap();
        let third_fib_row: &FibonacciRow<BabyBear> = (&*third_row).borrow();
        assert_eq!(third_fib_row.left.as_canonical_u32(), 2);
        assert_eq!(third_fib_row.right.as_canonical_u32(), 3);
        
        // Check last row
        let last_row = trace.row_slice(7).unwrap();
        let last_fib_row: &FibonacciRow<BabyBear> = (&*last_row).borrow();
        assert_eq!(last_fib_row.right.as_canonical_u32(), 34); // F(9)
    }
    
    #[test]
    fn test_trace_with_result() {
        let (trace, result) = generate_fibonacci_trace_with_result::<BabyBear>(1, 1, 8);
        
        assert_eq!(result, 34); // F(9) when starting from F(1)=1, F(2)=1
        assert_eq!(trace.height(), 8);
        
        // Verify the result matches the trace
        let last_row = trace.row_slice(7).unwrap();
        let last_fib_row: &FibonacciRow<BabyBear> = (&*last_row).borrow();
        assert_eq!(last_fib_row.right.as_canonical_u32(), result as u32);
    }
    
    #[test]
    #[should_panic(expected = "Number of steps must be a power of 2")]
    fn test_non_power_of_two_fails() {
        generate_fibonacci_trace::<BabyBear>(1, 1, 7); // 7 is not a power of 2
    }
    
    #[test]
    fn test_different_starting_values() {
        let trace = generate_fibonacci_trace::<BabyBear>(2, 3, 4);
        
        // Check first row
        let first_row = trace.row_slice(0).unwrap();
        let first_fib_row: &FibonacciRow<BabyBear> = (&*first_row).borrow();
        assert_eq!(first_fib_row.left.as_canonical_u32(), 2);
        assert_eq!(first_fib_row.right.as_canonical_u32(), 3);
        
        // Check second row: (3, 2+3=5)
        let second_row = trace.row_slice(1).unwrap();
        let second_fib_row: &FibonacciRow<BabyBear> = (&*second_row).borrow();
        assert_eq!(second_fib_row.left.as_canonical_u32(), 3);
        assert_eq!(second_fib_row.right.as_canonical_u32(), 5);
        
        // Check third row: (5, 3+5=8)
        let third_row = trace.row_slice(2).unwrap();
        let third_fib_row: &FibonacciRow<BabyBear> = (&*third_row).borrow();
        assert_eq!(third_fib_row.left.as_canonical_u32(), 5);
        assert_eq!(third_fib_row.right.as_canonical_u32(), 8);
    }
}