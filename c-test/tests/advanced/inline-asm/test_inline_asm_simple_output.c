// Test simple inline assembly with output operand
void putchar(int c);

int main() {
    int x = 10;
    int result;  // Uninitialized is fine - we'll write to it
    
    // Simple move from input to output
    asm("MOVE %0, %1" : "=r"(result) : "r"(x));
    
    if (result == 10) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    return 0;
}