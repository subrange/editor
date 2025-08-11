// Verify that pointer parameter preservation works
void putchar(int c);

void test_ptr(int *p) {
    putchar('A');  // This would clear registers before our fix
    putchar('0' + *p);  // This should still work - p should be preserved
}

int main() {
    int x = 5;
    test_ptr(&x);
    putchar('\n');
    return 0;
}