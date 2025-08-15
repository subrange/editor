void putchar(int c);

// Test global string initialization using __st8
char greeting[] = "Hello";

int main() {
    // Print each character of the greeting
    for (int i = 0; i < 5; i++) {
        putchar(greeting[i]);
    }
    putchar('\n');
    return 0;
}