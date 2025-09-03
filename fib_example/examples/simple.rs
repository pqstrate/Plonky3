//! Simple Fibonacci proof example
//! 
//! Run with: `cargo run --example simple`

use fib_example::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Simple Fibonacci Proof");
    println!("========================");
    
    // Prove that F(8) = 21 (starting from F(1)=1, F(2)=1) 
    prove_fibonacci(8, 21)?;
    
    println!("\n✅ Success! We proved knowledge of the 8th Fibonacci number.");
    
    Ok(())
}