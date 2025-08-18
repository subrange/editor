// Test basic qfixed arithmetic operations
#include <stdio.h>
#include <qfixed.h>

void print_fixed(struct q16_16_t *x) {
    // Print integer part
    int int_part = q_to_int(x);
    
    // Print fractional part (approximate as percentage of 65536)
    // frac / 65536 * 100 for percentage
    int frac_percent = (x->frac * 100) / 65536;
    
    if (int_part < 0) {
        // Handle negative numbers
        putchar('-');
        int_part = -int_part;
    }
    
    // Print integer part
    if (int_part >= 10) {
        putchar('0' + (int_part / 10));
    }
    putchar('0' + (int_part % 10));
    putchar('.');
    
    // Print fractional part (2 digits)
    putchar('0' + (frac_percent / 10));
    putchar('0' + (frac_percent % 10));
}

int main() {
    struct q16_16_t a, b, result;
    
    puts("Testing qfixed basic arithmetic:");
    
    // Test 1: Simple addition (2 + 3 = 5)
    puts("\nTest 1: 2 + 3");
    q_from_int(&a, 2);
    q_from_int(&b, 3);
    q_add(&result, &a, &b);
    puts("Result: ");
    print_fixed(&result);
    puts("");
    if (q_to_int(&result) == 5) {
        puts("PASS");
    } else {
        puts("FAIL");
    }
    
    // Test 2: Subtraction (5 - 2 = 3)
    puts("\nTest 2: 5 - 2");
    q_from_int(&a, 5);
    q_from_int(&b, 2);
    q_sub(&result, &a, &b);
    puts("Result: ");
    print_fixed(&result);
    puts("");
    if (q_to_int(&result) == 3) {
        puts("PASS");
    } else {
        puts("FAIL");
    }
    
    // Test 3: Multiplication (3 * 4 = 12)
    puts("\nTest 3: 3 * 4");
    q_from_int(&a, 3);
    q_from_int(&b, 4);
    q_mul(&result, &a, &b);
    puts("Result: ");
    print_fixed(&result);
    puts("");
    if (q_to_int(&result) == 12) {
        puts("PASS");
    } else {
        puts("FAIL");
    }
    
    // Test 4: Division (12 / 3 = 4)
    puts("\nTest 4: 12 / 3");
    q_from_int(&a, 12);
    q_from_int(&b, 3);
    q_div(&result, &a, &b);
    puts("Result: ");
    print_fixed(&result);
    puts("");
    if (q_to_int(&result) == 4) {
        puts("PASS");
    } else {
        puts("FAIL");
    }
    
    // Test 5: Fractional multiplication (1.5 * 2 = 3)
    puts("\nTest 5: 1.5 * 2");
    a.integer = 1;
    a.frac = 0x8000;  // 0.5 in fixed point
    q_from_int(&b, 2);
    q_mul(&result, &a, &b);
    puts("Result: ");
    print_fixed(&result);
    puts("");
    if (q_to_int(&result) == 3) {
        puts("PASS");
    } else {
        puts("FAIL");
    }
    
    // Test 6: Comparison tests
    puts("\nTest 6: Comparisons");
    q_from_int(&a, 5);
    q_from_int(&b, 3);
    
    if (q_gt(&a, &b)) {
        puts("5 > 3: PASS");
    } else {
        puts("5 > 3: FAIL");
    }
    
    if (q_lt(&b, &a)) {
        puts("3 < 5: PASS");
    } else {
        puts("3 < 5: FAIL");
    }
    
    q_from_int(&b, 5);
    if (q_eq(&a, &b)) {
        puts("5 == 5: PASS");
    } else {
        puts("5 == 5: FAIL");
    }
    
    puts("\nAll basic tests complete!");
    
    return 0;
}