//! # Fibonacci AIR (Arithmetic Intermediate Representation)
//! 
//! The AIR defines the constraints that must be satisfied by a valid Fibonacci computation.
//! This is the "circuit" that defines what computations are allowed.

use core::borrow::Borrow;
use p3_air::{Air, AirBuilder, AirBuilderWithPublicValues, BaseAir};
use p3_matrix::Matrix;
use crate::{FibonacciRow, NUM_FIBONACCI_COLS};

/// The Fibonacci AIR defines the computation constraints
/// 
/// ## Constraints:
/// 1. **Boundary constraints** (applied to first and last rows):
///    - First row: left = public_input[0], right = public_input[1] 
///    - Last row: right = public_input[2] (the result we're proving)
/// 
/// 2. **Transition constraints** (applied between consecutive rows):
///    - next.left = current.right (shift left)
///    - next.right = current.left + current.right (Fibonacci recurrence)
/// 
/// ## Public Values:
/// - public_values[0]: Initial left value (usually 1)
/// - public_values[1]: Initial right value (usually 1) 
/// - public_values[2]: Final result we're proving knowledge of
#[derive(Clone)]
pub struct FibonacciAir {
    // AIR can be stateless for Fibonacci, but we keep it for extensibility
}

impl FibonacciAir {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for FibonacciAir {
    fn default() -> Self {
        Self::new()
    }
}

/// BaseAir implementation - defines basic properties of our computation
impl<F> BaseAir<F> for FibonacciAir {
    /// Number of columns in our execution trace
    fn width(&self) -> usize {
        NUM_FIBONACCI_COLS
    }
}

/// Air implementation - defines the actual constraints
impl<AB: AirBuilderWithPublicValues> Air<AB> for FibonacciAir {
    /// This is where we define ALL the constraints that must be satisfied
    /// by a valid Fibonacci computation
    fn eval(&self, builder: &mut AB) {
        // Get access to the execution trace (main columns)
        let main = builder.main();
        
        // Get access to public values (inputs/outputs that are known to verifier)
        let public_values = builder.public_values();
        
        // Extract public values
        let initial_left = public_values[0];   // Usually F(1) = 1
        let initial_right = public_values[1];  // Usually F(2) = 1  
        let final_result = public_values[2];   // The F(n) we're proving knowledge of
        
        // Get current and next rows from the trace
        let (current_row, next_row) = (
            main.row_slice(0).expect("Matrix must have at least one row"),
            main.row_slice(1).expect("Matrix must have at least two rows for transitions"),
        );
        
        // Cast to our structured row type
        let current: &FibonacciRow<AB::Var> = (*current_row).borrow();
        let next: &FibonacciRow<AB::Var> = (*next_row).borrow();
        
        // === BOUNDARY CONSTRAINTS ===
        // These constraints are only applied to specific rows (first/last)
        
        // First row constraints: Initialize with public values
        let mut when_first_row = builder.when_first_row();
        when_first_row.assert_eq(current.left.clone(), initial_left);
        when_first_row.assert_eq(current.right.clone(), initial_right);
        
        // Last row constraint: Final value must match expected result
        builder.when_last_row().assert_eq(current.right.clone(), final_result);
        
        // === TRANSITION CONSTRAINTS ===
        // These constraints are applied between every pair of consecutive rows
        
        let mut when_transition = builder.when_transition();
        
        // Fibonacci recurrence relation:
        // F(n+1) = F(n-1) + F(n)
        // 
        // In terms of our columns:
        // - current.left = F(n-1), current.right = F(n)
        // - next.left = F(n), next.right = F(n+1)
        // 
        // So our constraints are:
        // 1. next.left = current.right  (shift: F(n) becomes the new F(n-1))
        // 2. next.right = current.left + current.right  (F(n+1) = F(n-1) + F(n))
        
        when_transition.assert_eq(next.left.clone(), current.right.clone());
        when_transition.assert_eq(next.right.clone(), current.left.clone() + current.right.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use p3_baby_bear::BabyBear;
    
    #[test]
    fn test_fibonacci_air_properties() {
        let air = FibonacciAir::new();
        assert_eq!(air.width(), NUM_FIBONACCI_COLS);
    }
    
    #[test]
    fn test_fibonacci_row_borrow() {
        let values = vec![BabyBear::new(1), BabyBear::new(1)];
        let row: &FibonacciRow<BabyBear> = values.as_slice().borrow();
        assert_eq!(row.left, BabyBear::new(1));
        assert_eq!(row.right, BabyBear::new(1));
    }
}