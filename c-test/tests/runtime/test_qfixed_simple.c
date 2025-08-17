// Simplest Q16.16 test to verify basic functionality
#include "qfixed.h"
#include "test_assert.h"

int main() {
    // Test basic integer conversion
    q16_16_t x = q_from_int(5);
    TEST_ASSERT_EQ(x.integer, 5);
    TEST_ASSERT_EQ(x.frac, 0);
    
    // Test constants
    q16_16_t half = Q16_16_HALF;
    TEST_ASSERT_EQ(half.integer, 0);
    TEST_ASSERT_EQ(half.frac, 0x8000);
    
    // Test simple addition
    q16_16_t one = Q16_16_ONE;
    q16_16_t two = q_add(one, one);
    TEST_ASSERT_EQ(two.integer, 2);
    
    TEST_COMPLETE();
    return 0;
}