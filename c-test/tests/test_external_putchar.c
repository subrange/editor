// Test using external putchar from runtime library
// Declare putchar as external
extern void putchar(int c);

int main() {
    putchar('H');
    putchar('i');
    putchar('\n');
    return 0;
}