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
    
    // Test with string concatenation across multiple lines
    asm(
        "LI T0, 10; "
        "LI T1, 5; "
    );
    
    asm(
        "SUB T2, T0, T1; "  // 10 - 5 = 5
        "LI T3, 5; "
        "BEQ T2, T3, sub_ok; "
        "LI T4, 78; "         // 'N'
        "STORE T4, R0, R0; "
        "BEQ R0, R0, sub_done; "
        "sub_ok: "
        "LI T4, 89; "         // 'Y'
        "STORE T4, R0, R0; "
        "sub_done: "
    );
    
    // Output newline
    asm("LI T1, 10; STORE T1, R0, R0");
    
    return 0;
}
