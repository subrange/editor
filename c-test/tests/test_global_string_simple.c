void putchar(int c);

// Test global string initialization - simple version
// Just check that the string is stored correctly
char greeting[] = "Hi";

int main() {
    putchar(greeting[0]); // 'H'
    putchar(greeting[1]); // 'i'
    putchar('\n');
    return 0;
}