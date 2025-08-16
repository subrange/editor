// Test multiline inline assembly syntax support
void putchar(int c);

int main() {
    // Test 1: Single line with semicolons
    asm("LI T0, 1; LI T1, 2; ADD T2, T0, T1");
    
    // Test 2: Adjacent string concatenation
    asm(
        "LI T3, 3; "
        "LI T4, 4; "
        "ADD T1, T3, T4"
    );
    
    // Test 3: Multiple asm statements
    asm("LI T1, 5");
    asm("LI T2, 6");
    asm("ADD T3, T1, T2");
    
    // If we get here, the syntax parsing worked
    putchar('O');
    putchar('K');
    putchar('\n');
    
    return 0;
}
