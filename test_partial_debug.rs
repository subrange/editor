// Quick test to understand the behavior
fn main() {
    // Simulate the logic from setup_call_args
    let mut arg_reg_idx = 0;
    
    // Scalar 1
    if arg_reg_idx < 4 {
        println!("Arg 0 (scalar) -> A{}", arg_reg_idx);
        arg_reg_idx += 1;
    }
    
    // Scalar 2
    if arg_reg_idx < 4 {
        println!("Arg 1 (scalar) -> A{}", arg_reg_idx);
        arg_reg_idx += 1;
    }
    
    // Scalar 3
    if arg_reg_idx < 4 {
        println!("Arg 2 (scalar) -> A{}", arg_reg_idx);
        arg_reg_idx += 1;
    }
    
    println!("After 3 scalars, arg_reg_idx = {}", arg_reg_idx);
    
    // Fat pointer
    if arg_reg_idx + 1 < 4 {
        println!("Arg 3 (fat ptr) -> A{}, A{}", arg_reg_idx, arg_reg_idx + 1);
        arg_reg_idx += 2;
    } else {
        println!("Arg 3 (fat ptr) -> stack");
    }
    
    // Scalar 4
    if arg_reg_idx < 4 {
        println!("Arg 4 (scalar) -> A{}", arg_reg_idx);
        arg_reg_idx += 1;
    } else {
        println!("Arg 4 (scalar) -> stack");
    }
}