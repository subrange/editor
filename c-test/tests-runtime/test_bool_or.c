void putchar(int c);

int main() {
    int i = 1;
    int val = 20;
    
    // This exact condition from Test 4
    if ((i == 0 && val == 10) || (i == 1 && val == 20)) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}