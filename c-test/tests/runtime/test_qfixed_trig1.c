// Test qfixed trigonometric functions
#include <stdio.h>
#include <qfixed.h>

void print_fixed(struct q16_16_t *x) {
    // Handle sign
    if (x->integer < 0 || (x->integer == -1 && x->frac > 0)) {
        putchar('-');
        // Negate for display
        struct q16_16_t neg_x;
        neg_x.integer = -x->integer;
        if (x->frac > 0) {
            neg_x.integer--;
            neg_x.frac = 0x10000 - x->frac;
        } else {
            neg_x.frac = 0;
        }
        x = &neg_x;
    }
    
    // Print integer part
    int int_part = x->integer;
    if (int_part < 0) int_part = -int_part;
    
    putchar('0' + (int_part % 10));
    putchar('.');
    
    // Print fractional part (2 digits)
    int frac_percent = (x->frac * 100) / 65536;
    putchar('0' + (frac_percent / 10));
    putchar('0' + (frac_percent % 10));
}

int main() {
    struct q16_16_t angle, result;
    
    puts("Testing qfixed trigonometry:");
    
    // Test sin(0) = 0
    puts("\nTest sin(0):");
    angle.integer = 0;
    angle.frac = 0;
    q_sin(&result, &angle);
    puts("sin(0) = ");
    print_fixed(&result);
    puts("");
    
    // Test cos(0) = 1
    puts("\nTest cos(0):");
    q_cos(&result, &angle);
    puts("cos(0) = ");
    print_fixed(&result);
    puts("");
    
    // Test sin(1) ≈ 0.84
    puts("\nTest sin(1):");
    angle.integer = 1;
    angle.frac = 0;
    q_sin(&result, &angle);
    puts("sin(1) = ");
    print_fixed(&result);
    puts("");
    
    // Test cos(1) ≈ 0.54
    puts("\nTest cos(1):");
    q_cos(&result, &angle);
    puts("cos(1) = ");
    print_fixed(&result);
    puts("");
    
    // Test sin(2) ≈ 0.91
    puts("\nTest sin(2):");
    angle.integer = 2;
    angle.frac = 0;
    q_sin(&result, &angle);
    puts("sin(2) = ");
    print_fixed(&result);
    puts("");
    
    // Test cos(2) ≈ -0.42
    puts("\nTest cos(2):");
    q_cos(&result, &angle);
    puts("cos(2) = ");
    print_fixed(&result);
    puts("");
    
    // Test sin(3) ≈ 0 (close to π)
    puts("\nTest sin(3):");
    angle.integer = 3;
    angle.frac = 0;
    q_sin(&result, &angle);
    puts("sin(3) = ");
    print_fixed(&result);
    puts("");
    
    // Test cos(3) ≈ -1
    puts("\nTest cos(3):");
    q_cos(&result, &angle);
    puts("cos(3) = ");
    print_fixed(&result);
    puts("");
    
    // Test negative angle sin(-1)
    puts("\nTest sin(-1):");
    angle.integer = -1;
    angle.frac = 0;
    q_sin(&result, &angle);
    puts("sin(-1) = ");
    print_fixed(&result);
    puts("");
    
    // Test negative angle cos(-1)
    puts("\nTest cos(-1):");
    q_cos(&result, &angle);
    puts("cos(-1) = ");
    print_fixed(&result);
    puts("");
    
    puts("\nTrigonometry tests complete!");
    
    return 0;
}