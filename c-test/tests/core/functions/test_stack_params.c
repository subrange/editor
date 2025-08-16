void putchar(int c);

void test_params(int *p1, int *p2, int a, char b) {
    putchar(b);
    putchar('\n');
}

int main() {
    int x = 1;
    int y = 2;
    test_params(&x, &y, 42, 'Z');
    return 0;
}