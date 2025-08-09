extern void putchar(int c);

int puts(char *s) {
    putchar(s[0]);
    putchar(s[1]);
    putchar(s[2]);
    putchar('\n');
    return 3;
}

int main() {
    puts("XYZ");
    return 0;
}