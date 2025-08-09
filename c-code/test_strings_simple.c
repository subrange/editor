void putchar(int c);
void puts(char *s);

// Simple puts implementation
void puts(char *s) {
    while (*s) {
        putchar(*s);
        s++;
    }
}

int main() {
    puts("Hello!\n");
    return 0;
}