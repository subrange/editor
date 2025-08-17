// Test inline assembly with input/output operands
void putchar(int c);

int main() {
    int x = 10;
    int y = 20;
    int sum = 0;
    int product = 0;
    
    // Test 1: Basic addition with extended syntax
    asm(
        "ADD %0, %1, %2"
        : "=r"(sum)         // Output: sum gets the result
        : "r"(x), "r"(y)    // Inputs: x and y
    );
    
    // Test 2: Multiplication with multiline assembly
    asm(
        "MUL T4, %1, %2; "   // Multiply inputs
        "MOVE %0, T4"        // Move result to output
        : "=r"(product)      // Output
        : "r"(x), "r"(y)     // Inputs
        : "T4"               // Clobber T4 register
    );
    
    // Test 3: In-place modification (input/output operand)
    int counter = 5;
    asm(
        "ADDI %0, %0, 1"     // Increment by 1
        : "+r"(counter)      // Input and output (read-write)
    );
    
    // Test 4: Complex multiline with multiple operations
    int a = 3;
    int b = 4;
    int c = 0;
    asm(
        "MUL T5, %1, %1; "   // a * a
        "MUL T6, %2, %2; "   // b * b  
        "ADD %0, T5, T6"     // a^2 + b^2
        : "=r"(c)            // Output: c = a^2 + b^2
        : "r"(a), "r"(b)     // Inputs
        : "T5", "T6"         // Clobbers
    );
    
    // Verify results
    if (sum == 30) {         // 10 + 20 = 30
        putchar('Y');
    } else {
        putchar('N');
    }
    
    if (product == 200) {    // 10 * 20 = 200
        putchar('Y');
    } else {
        putchar('N');
    }
    
    if (counter == 6) {      // 5 + 1 = 6
        putchar('Y');
    } else {
        putchar('N');
    }
    
    if (c == 25) {           // 3^2 + 4^2 = 9 + 16 = 25
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}