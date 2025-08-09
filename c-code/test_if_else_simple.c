// Simple if-else test to verify branching
void putchar(int c);

int main() {
    // Test 1: Simple if with true condition
    putchar('1');
    putchar(':');
    if (5 > 3) {
        putchar('T');
    } else {
        putchar('F');
    }
    putchar(10);
    
    // Test 2: Simple if with false condition
    putchar('2');
    putchar(':');
    if (2 > 5) {
        putchar('T');
    } else {
        putchar('F');
    }
    putchar(10);
    
    // Test 3: Equality test (true)
    putchar('3');
    putchar(':');
    if (7 == 7) {
        putchar('T');
    } else {
        putchar('F');
    }
    putchar(10);
    
    // Test 4: Not equal (false case)
    putchar('4');
    putchar(':');
    if (3 != 3) {
        putchar('T');
    } else {
        putchar('F');
    }
    putchar(10);
    
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
    putchar(10);
    
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
    putchar(10);
    
    // Test 7: Multiple statements in branches
    putchar('7');
    putchar(':');
    if (1 < 2) {
        putchar('O');
        putchar('K');
    } else {
        putchar('N');
        putchar('O');
    }
    putchar(10);
    
    // Test 8: Empty else branch
    putchar('8');
    putchar(':');
    if (3 > 2) {
        putchar('Y');
    }
    putchar(10);
    
    // Test 9: Variables in conditions
    putchar('9');
    putchar(':');
    int a = 5;
    int b = 3;
    if (a > b) {
        putchar('T');
    } else {
        putchar('F');
    }
    putchar(10);
    
    // Test 10: Zero/non-zero as boolean
    putchar('A');
    putchar(':');
    if (5) {  // Non-zero is true
        putchar('T');
    } else {
        putchar('F');
    }
    putchar(10);
    
    putchar('B');
    putchar(':');
    if (0) {  // Zero is false
        putchar('T');
    } else {
        putchar('F');
    }
    putchar(10);
    
    return 0;
}