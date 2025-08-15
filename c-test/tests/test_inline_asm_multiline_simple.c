// Test multiline inline assembly with adjacent string concatenation
void putchar(int c);

int main() {
    // Initialize some values in registers using multiline assembly
    // This demonstrates how to write readable multiline assembly
    asm(
        "LI T0, 10; "          // Load 10 into T0
        "LI T1, 20; "          // Load 20 into T1
        "ADD T2, T0, T1; "     // Add T0 and T1, store in T2
        "LI T3, 30; "          // Load expected value
        "BEQ T2, T3, equal; "  // Branch if equal
        "LI T4, 78; "          // 'N' for not equal
        "PUSH T4; CALL putchar; POP T4; "   // Output 'N'
        "BEQ R0, R0, done; "   // Jump to done
        "equal: "
        "LI T4, 89; "          // 'Y' for equal
        "STORE T4, R0, R0; "   // Output 'Y'
        "done: "
    );
    
    // Test multiplication
    asm(
        "LI T5, 6; "
        "LI T6, 7; "
        "MUL T7, T5, T6; "     // 6 * 7 = 42
        "LI T8, 42; "
        "BEQ T7, T8, mult_ok; "
        "LI T9, 78; "          // 'N'
        "STORE T9, R0, R0; "
        "BEQ R0, R0, mult_done; "
        "mult_ok: "
        "LI T9, 89; "          // 'Y'
        "STORE T9, R0, R0; "
        "mult_done: "
    );
    
    // Test with string concatenation across multiple lines
    asm(
        "LI T10, 10; "
        "LI T11, 5; "
    );
    
    asm(
        "SUB T12, T10, T11; "  // 10 - 5 = 5
        "LI T13, 5; "
        "BEQ T12, T13, sub_ok; "
        "LI T14, 78; "         // 'N'
        "STORE T14, R0, R0; "
        "BEQ R0, R0, sub_done; "
        "sub_ok: "
        "LI T14, 89; "         // 'Y'
        "STORE T14, R0, R0; "
        "sub_done: "
    );
    
    // Output newline
    asm("LI T15, 10; STORE T15, R0, R0");
    
    return 0;
}