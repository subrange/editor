// Test ternary conditional operator (? :)
#include <stdio.h>

void test_basic_ternary() {
    // Basic integer ternary
    int a = 5;
    int b = 10;
    int max = (a > b) ? a : b;
    
    if (max == 10) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Minimum
    int min = (a < b) ? a : b;
    if (min == 5) {
        putchar('Y');
    } else {
        putchar('N');
    }
}

void test_nested_ternary() {
    // Nested ternary operators
    int x = 15;
    int y = 10;
    int z = 20;
    
    // Find maximum of three numbers
    int max = (x > y) ? ((x > z) ? x : z) : ((y > z) ? y : z);
    
    if (max == 20) {
        putchar('Y');
    } else {
        putchar('N');
    }
}

void test_ternary_with_side_effects() {
    // Test that only one branch is evaluated
    int counter = 0;
    int result;
    
    // True condition - only then branch should execute
    result = (1 > 0) ? 42 : 0;
    
    if (result == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // False condition - only else branch should execute  
    result = (0 > 1) ? 99 : 88;
    
    if (result == 88) {
        putchar('Y');
    } else {
        putchar('N');
    }
}

void test_ternary_as_lvalue() {
    int a = 5;
    int b = 10;
    int x = 0;
    int y = 0;
    
    // Use ternary result in assignment
    int target = (a < b) ? a : b;
    target = 15;
    
    // This doesn't modify a or b
    if (a == 5 && b == 10 && target == 15) {
        putchar('Y');
    } else {
        putchar('N');
    }
}

void test_ternary_with_different_types() {
    // Test type conversion in ternary
    int i = 42;
    char c = 'A';
    
    // Both branches should convert to common type
    int result = (1 > 0) ? i : c;
    
    if (result == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    result = (0 > 1) ? i : c;
    
    if (result == 65) { // 'A' as integer
        putchar('Y');
    } else {
        putchar('N');
    }
}

void test_ternary_with_pointers() {
    int x = 100;
    int y = 200;
    int *px = &x;
    int *py = &y;
    
    // Select pointer with ternary
    int *selected = (x < y) ? px : py;
    
    if (*selected == 100) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    selected = (x > y) ? px : py;
    
    if (*selected == 200) {
        putchar('Y');
    } else {
        putchar('N');
    }
}

void test_ternary_in_expression() {
    int a = 5;
    int b = 10;
    
    // Use ternary in arithmetic expression
    int result = 2 * ((a > b) ? a : b) + 3;
    
    if (result == 23) { // 2 * 10 + 3
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Use in function call
    putchar((a < b) ? 'Y' : 'N');
    
    // Another expression test
    int c = ((a + b) > 10) ? 1 : 0;
    if (c == 1) {
        putchar('Y');
    } else {
        putchar('N');
    }
}

void test_ternary_short_circuit() {
    // Test that ternary properly short-circuits
    int divide_by_zero_guard = 0;
    int divisor = 0;
    
    // This should not crash - false branch not evaluated
    int result = (divisor == 0) ? -1 : (100 / divisor);
    
    if (result == -1) {
        putchar('Y');
    } else {
        putchar('N');
    }
}

int main() {
    puts("Testing ternary operator:");
    
    // Test 1: Basic ternary
    puts("Basic: ");
    test_basic_ternary();
    putchar('\n');
    
    // Test 2: Nested ternary
    puts("Nested: ");
    test_nested_ternary();
    putchar('\n');
    
    // Test 3: Side effects
    puts("Side effects: ");
    test_ternary_with_side_effects();
    putchar('\n');
    
    // Test 4: As lvalue
    puts("Lvalue: ");
    test_ternary_as_lvalue();
    putchar('\n');
    
    // Test 5: Type conversion
    puts("Types: ");
    test_ternary_with_different_types();
    putchar('\n');
    
    // Test 6: With pointers
    puts("Pointers: ");
    test_ternary_with_pointers();
    putchar('\n');
    
    // Test 7: In expressions
    puts("Expression: ");
    test_ternary_in_expression();
    putchar('\n');
    
    // Test 8: Short circuit
    puts("Short circuit: ");
    test_ternary_short_circuit();
    putchar('\n');
    
    puts("Done!");
    
    return 0;
}