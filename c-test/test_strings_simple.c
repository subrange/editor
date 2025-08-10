void putchar(int c);

// Simple puts implementation
void puts(char *s) {
    while (*s) {
        putchar(*s);
        s = s + 1;
    }
}

int main() {
    puts("Hello!\n");
    return 0;
}