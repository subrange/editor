// Test puts with string literals (which are global)
void putchar(int c);

int puts(char *str) {
    int i = 0;
    while (str[i] != '\0') {
        putchar(str[i]);
        i = i + 1;
    }
    putchar('\n');
    return i;
}

int main() {
    puts("ABC");
    puts("Hello");
    puts("!");
    return 0;
}