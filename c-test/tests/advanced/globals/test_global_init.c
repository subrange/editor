// Test that globals are initialized

int g1 = 65;  // 'A'
int g2 = 66;  // 'B'

void putchar(int c);

int main() {
    putchar(g1);
    putchar(g2);
    putchar('\n');
    return 0;
}

