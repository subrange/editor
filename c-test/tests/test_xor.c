void putchar(int c);

void main() {
  int i = 3 ^ 2; // XOR operation
    if (i == 1) {
        putchar('Y'); // Test passed
    } else {
        putchar('N'); // Test failed
    }
}