// Test program without _start (uses crt0.asm)
// Compile this without _start and link with crt0.pobj

void putchar(int c);

int main() {
    // Direct MMIO output since putchar is special-cased
    int i = 2 * 3;

    if (i == 6) {
        putchar('Y');  // Test passes
    } else {
        putchar('N');  // Test fails
    }

    return 0;
}