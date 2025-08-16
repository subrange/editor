// Debug version to test puts
void putchar(int c);

int puts_simple(char *str) {
    putchar(str[0]);
    putchar(str[1]);
    putchar(str[2]);
    putchar('\n');
    return 3;
}

int main() {
    puts_simple("ABC");
    return 0;
}