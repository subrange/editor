void putchar(int c);

void test_char(char ch) {
    putchar(ch);
}

int main() {
    test_char('X');
    putchar('\n');
    return 0;
}