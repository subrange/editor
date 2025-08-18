// Debug qfixed multiplication
#include <stdio.h>
#include <qfixed.h>

int main() {
    struct q16_16_t a, b, result;
    
    // Test 1.5 * 2
    a.integer = 1;
    a.frac = 0x8000;  // 0.5
    
    b.integer = 2;
    b.frac = 0;
    
    puts("Testing 1.5 * 2:");
    puts("a.integer = 1");
    puts("a.frac = 0x8000");
    puts("b.integer = 2");
    puts("b.frac = 0");
    
    q_mul(&result, &a, &b);
    
    puts("Result:");
    puts("result.integer = ");
    putchar('0' + result.integer);
    puts("");
    puts("result.frac = ");
    // Print frac in hex (simplified)
    if (result.frac == 0) {
        puts("0x0000");
    } else if (result.frac == 0x8000) {
        puts("0x8000");
    } else {
        puts("other");
    }
    
    // Check what we get
    int int_part = q_to_int(&result);
    puts("q_to_int = ");
    putchar('0' + int_part);
    puts("");
    
    // Expected: integer = 3, frac = 0
    if (result.integer == 3 && result.frac == 0) {
        puts("PASS");
    } else {
        puts("FAIL");
    }
    
    return 0;
}