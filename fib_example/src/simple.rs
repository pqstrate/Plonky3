//! Simple test to understand the compilation issues

use p3_baby_bear::BabyBear;
use p3_field::PrimeField32;

pub fn test_baby_bear() {
    let value = BabyBear::new(42);
    println!("BabyBear value: {}", value.as_canonical_u32());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_baby_bear_creation() {
        test_baby_bear();
    }
}