// Test multiline inline assembly without labels
void putchar(int c);

int main() {
    // Test 1: Simple calculations with semicolons
    asm("LI T0, 10; LI T1, 20; ADD T2, T0, T1");
    
    // Test 2: Adjacent string concatenation
    asm(
        "LI T3, 3; "
        "LI T4, 4; "
        "MUL T5, T3, T4"
    );
    
    // Test 3: More math operations
    asm(
        "LI T6, 100; "
        "LI T7, 50; "
        "SUB T8, T6, T7; "
        "LI T9, 2; "
        "DIV T10, T8, T9"
    );
    
    // If we compiled successfully, output OK
    putchar('O');
    putchar('K');
    putchar('\n');
    
    return 0;
}