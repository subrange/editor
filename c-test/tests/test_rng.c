// Test RNG functionality

void putchar(int c);

int main() {
    // Test that we can output through putchar
    putchar('R');
    putchar('N');
    putchar('G');
    putchar(':');
    
    // For now, just output OK since we need to implement
    // proper MMIO access through the runtime
    putchar('O');
    putchar('K');
    putchar('\n');
    
    return 0;
}