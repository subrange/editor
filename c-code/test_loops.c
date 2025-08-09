// Test loops with proper branching
void putchar(int c);

int main() {
    // Test 1: Simple while loop
    putchar('W');
    putchar(':');
    int i = 0;
    while (i < 3) {
        putchar('0' + i);
        i = i + 1;
    }
    putchar(10);
    
    // Test 2: For loop
    putchar('F');
    putchar(':');
    for (int j = 0; j < 3; j = j + 1) {
        putchar('A' + j);
    }
    putchar(10);
    
    // Test 3: Do-while loop
    putchar('D');
    putchar(':');
    int k = 0;
    do {
        putchar('X' + k);
        k = k + 1;
    } while (k < 3);
    putchar(10);
    
    // Test 4: Nested loops
    putchar('N');
    putchar(':');
    for (int x = 0; x < 2; x = x + 1) {
        for (int y = 0; y < 2; y = y + 1) {
            putchar('0' + x);
            putchar('0' + y);
            putchar(' ');
        }
    }
    putchar(10);
    
    // Test 5: Break (if supported)
    putchar('B');
    putchar(':');
    for (int m = 0; m < 5; m = m + 1) {
        if (m == 2) {
            break;
        }
        putchar('0' + m);
    }
    putchar(10);
    
    // Test 6: Continue (if supported)
    putchar('C');
    putchar(':');
    for (int n = 0; n < 5; n = n + 1) {
        if (n == 2) {
            continue;
        }
        putchar('0' + n);
    }
    putchar(10);
    
    return 0;
}