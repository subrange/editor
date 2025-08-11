void putchar(int c);

int add(int a, int b) {
    return a + b;
}

int main() {
    int result = add(2, 3);
    putchar('0' + result);
    putchar('\n');
    return 0;
}