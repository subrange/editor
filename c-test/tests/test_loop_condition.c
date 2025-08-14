void putchar(int c);

int main() {
    int i = 3;
    
    // Test the loop condition manually
    if (i > 1) putchar('Y'); else putchar('N');
    putchar('\n');
    
    i = i - 1; // i = 2
    if (i > 1) putchar('Y'); else putchar('N');
    putchar('\n');
    
    i = i - 1; // i = 1
    if (i > 1) putchar('Y'); else putchar('N');
    putchar('\n');
    
    // Now test in actual loop
    putchar('L'); putchar(':'); putchar('\n');
    i = 3;
    while (i > 1) {
        putchar('0' + i);
        putchar('\n');
        i = i - 1;
    }
    putchar('E'); putchar('\n');
    
    return 0;
}