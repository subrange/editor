// Basic Q16.16 fixed-point tests
#include <stdio.h>
#include "qfixed.h"
#include "test_assert.h"

// Helper to print Q16.16 value (simplified)
void print_q(q16_16_t x) {
    // Print integer part
    int int_part = x.integer;
    if (int_part < 0) {
        putchar('-');
        int_part = -int_part;
        // Handle negative fractional part
        if (x.frac > 0) {
            int_part--;
            x.frac = 0x10000 - x.frac;
        }
    }
    
    // Print integer digits
    if (int_part >= 10) {
        putchar('0' + (int_part / 10));
    }
    putchar('0' + (int_part % 10));
    
    // Print decimal point and first 2 fractional digits
    putchar('.');
    
    // Convert fraction to decimal (approximate)
    unsigned int decimal = (x.frac * 100) >> 16;
    putchar('0' + (decimal / 10));
    putchar('0' + (decimal % 10));
}

int main() {
    // Test 1: Basic addition
    q16_16_t a = q_from_int(3);
    q16_16_t b = Q16_16_HALF;  // 0.5
    q16_16_t c = q_add(a, b);  // Should be 3.5
    
    TEST_ASSERT(c.integer == 3 && c.frac == 0x8000);  // Addition works
    
    // Test 2: Subtraction
    q16_16_t d = q_from_int(5);
    q16_16_t e = q_from_int(2);
    q16_16_t f = q_sub(d, e);  // Should be 3
    
    TEST_ASSERT_EQ(f.integer, 3);
    TEST_ASSERT_EQ(f.frac, 0);  // Subtraction works
    
    // Test 3: Multiplication
    q16_16_t g = q_from_int(4);
    q16_16_t h = Q16_16_HALF;  // 0.5
    q16_16_t i = q_mul(g, h);  // Should be 2
    
    TEST_ASSERT(i.integer == 2 && i.frac == 0);  // Multiplication works
    
    // Test 4: Division
    q16_16_t j = q_from_int(10);
    q16_16_t k = q_from_int(4);
    q16_16_t l = q_div(j, k);  // Should be 2.5
    
    TEST_ASSERT(l.integer == 2 && l.frac == 0x8000);  // Division works
    
    // Test 5: Comparison
    q16_16_t m = q_from_int(5);
    q16_16_t n = q_from_int(3);
    
    TEST_ASSERT(q_gt(m, n) && q_lt(n, m));  // Comparison works
    
    // Test 6: Negative numbers
    q16_16_t o = q_from_int(-3);
    q16_16_t p = q_from_int(2);
    q16_16_t q = q_add(o, p);  // Should be -1
    
    TEST_ASSERT(q.integer == -1 && q.frac == 0);  // Negative handling works
    
    // Test 7: Square root
    q16_16_t r = q_from_int(4);
    q16_16_t s = q_sqrt(r);  // Should be 2
    
    TEST_ASSERT(s.integer == 2 && s.frac < 0x1000);  // Square root works (allow small error)
    
    // Test 8: Reciprocal
    q16_16_t t = q_from_int(2);
    q16_16_t u = q_reciprocal(t);  // Should be 0.5
    
    TEST_ASSERT_IN_RANGE(u.frac, 0x7F00, 0x8100);  // Reciprocal works (allow small error)
    
    TEST_COMPLETE();
    return 0;
}