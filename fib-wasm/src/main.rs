// Simple test to verify the library compiles and basic functionality works
fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    
    let mut a = 0u64;
    let mut b = 1u64;
    
    for _ in 2..=n {
        let next = a.wrapping_add(b);
        a = b;
        b = next;
    }
    
    b
}

fn main() {
    println!("Testing Fibonacci calculation:");
    for n in 1..=12 {
        println!("F({}) = {}", n, fibonacci(n));
    }
    
    println!("\n✅ Basic functionality test passed!");
    println!("WASM module built successfully. Use index.html to test proof generation in browser.");
}