// Test complex boolean expressions with multiple conditions
// This isolates the issue found in test_pointer_phi

void putchar(int c);

int main() {
    // Test 1: Simple AND
    int a = 1;
    int b = 1;
    if (a == 1 && b == 1) {
        putchar('1');
    } else {
        putchar('N');
    }

    // Test 2: Simple OR
    a = 1;
    b = 0;
    if (a == 1 || b == 1) {
        putchar('2');
    } else {
        putchar('N');
    }

    // Test 3: Complex expression like in test_pointer_phi
    // (i == 0 && val == 10) || (i == 1 && val == 20)
    int i = 0;
    int val = 10;
    if ((i == 0 && val == 10) || (i == 1 && val == 20)) {
        putchar('3');
    } else {
        putchar('N');
    }

    // Test 4: Same but with i=1, val=20
    i = 1;
    val = 20;
    if ((i == 0 && val == 10) || (i == 1 && val == 20)) {
        putchar('4');
    } else {
        putchar('N');
    }

    // Test 5: Should fail - neither condition true
    i = 0;
    val = 20;
    if ((i == 0 && val == 10) || (i == 1 && val == 20)) {
        putchar('F');
    } else {
        putchar('5');
    }

    putchar('\n');
    return 0;
}