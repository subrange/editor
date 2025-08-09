// Comprehensive if-else test
void putchar(int c);

int main() {
    // Test 1: Simple if-else
    putchar('1');
    putchar(':');
    if (5 > 3) {
        putchar('T');
    } else {
        putchar('F');
    }
    putchar(' ');
    
    // Test 2: False condition
    putchar('2');
    putchar(':');
    if (2 > 5) {
        putchar('T');
    } else {
        putchar('F');
    }
    putchar(' ');
    
    // Test 3: Equality test
    putchar('3');
    putchar(':');
    if (7 == 7) {
        putchar('T');
    } else {
        putchar('F');
    }
    putchar(' ');
    
    // Test 4: Not equal
    putchar('4');
    putchar(':');
    if (3 != 3) {
        putchar('T');
    } else {
        putchar('F');
    }
    putchar(' ');
    
    // Test 5: Nested if-else
    putchar('5');
    putchar(':');
    if (10 > 5) {
        if (3 < 7) {
            putchar('A');
        } else {
            putchar('B');
        }
    } else {
        putchar('C');
    }
    putchar(' ');
    
    // Test 6: else-if chain
    putchar('6');
    putchar(':');
    int x = 2;
    if (x == 1) {
        putchar('1');
    } else if (x == 2) {
        putchar('2');
    } else if (x == 3) {
        putchar('3');
    } else {
        putchar('X');
    }
    putchar(' ');
    
    // Test 7: Complex condition
    putchar('7');
    putchar(':');
    if (5 > 3 && 2 < 4) {
        putchar('T');
    } else {
        putchar('F');
    }
    putchar(' ');
    
    // Test 8: OR condition
    putchar('8');
    putchar(':');
    if (2 > 5 || 3 < 7) {
        putchar('T');
    } else {
        putchar('F');
    }
    putchar(' ');
    
    // Test 9: Side effects in condition
    putchar('9');
    putchar(':');
    int y = 0;
    if (++y == 1) {
        putchar('T');
    } else {
        putchar('F');
    }
    putchar(' ');
    if (y == 1) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar(10);
    return 0;
}