// Test puts with string literals (which are global)
void puts(char *str);

int main() {
    puts("ABC");
    puts("Hello");
    puts("!");
    return 0;
}