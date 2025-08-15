// Test basic inline assembly
void putchar(int c);

int main() {
    // Test 1: Simple inline assembly without operands
    asm("LI T0, 72");  // 'H'
    
    // Test 2: Multi-line assembly (use semicolons as separators)
    asm("LI T1, 101; LI T2, 108; LI T3, 108; LI T4, 111");
    
    // For now, just output a simple test result
    putchar('O');
    putchar('K');
    putchar('\n');
    
    return 0;
}