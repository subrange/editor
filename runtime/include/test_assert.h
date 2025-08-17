#ifndef TEST_ASSERT_H
#define TEST_ASSERT_H

// Test assertion macros for RCT test suite
// These output 'Y' for pass and 'N' for fail, allowing tests to continue

void putchar(int c);

// Basic test assertion - outputs Y if true, N if false
#define TEST_ASSERT(expr) \
    do { \
        if (expr) { \
            putchar('Y'); \
        } else { \
            putchar('N'); \
        } \
    } while(0)

// Test equality assertion
#define TEST_ASSERT_EQ(actual, expected) \
    TEST_ASSERT((actual) == (expected))

// Test inequality assertion  
#define TEST_ASSERT_NE(actual, expected) \
    TEST_ASSERT((actual) != (expected))

// Test less than assertion
#define TEST_ASSERT_LT(actual, expected) \
    TEST_ASSERT((actual) < (expected))

// Test less than or equal assertion
#define TEST_ASSERT_LE(actual, expected) \
    TEST_ASSERT((actual) <= (expected))

// Test greater than assertion
#define TEST_ASSERT_GT(actual, expected) \
    TEST_ASSERT((actual) > (expected))

// Test greater than or equal assertion
#define TEST_ASSERT_GE(actual, expected) \
    TEST_ASSERT((actual) >= (expected))

// Test pointer equality
#define TEST_ASSERT_PTR_EQ(actual, expected) \
    TEST_ASSERT((void*)(actual) == (void*)(expected))

// Test pointer inequality
#define TEST_ASSERT_PTR_NE(actual, expected) \
    TEST_ASSERT((void*)(actual) != (void*)(expected))

// Test pointer is NULL
#define TEST_ASSERT_NULL(ptr) \
    TEST_ASSERT((void*)(ptr) == (void*)0)

// Test pointer is not NULL
#define TEST_ASSERT_NOT_NULL(ptr) \
    TEST_ASSERT((void*)(ptr) != (void*)0)

// Test boolean true
#define TEST_ASSERT_TRUE(expr) \
    TEST_ASSERT(!!(expr))

// Test boolean false
#define TEST_ASSERT_FALSE(expr) \
    TEST_ASSERT(!(expr))

// Test range (inclusive)
#define TEST_ASSERT_IN_RANGE(value, min, max) \
    TEST_ASSERT((value) >= (min) && (value) <= (max))

// Special assertions for Q16.16 fixed-point
#ifdef QFIXED_H

// Test Q16.16 equality
#define TEST_ASSERT_Q_EQ(actual, expected) \
    TEST_ASSERT(q_eq(actual, expected))

// Test Q16.16 approximate equality (within tolerance)
#define TEST_ASSERT_Q_NEAR(actual, expected, tolerance) \
    TEST_ASSERT(q_abs(q_sub(actual, expected)).integer == 0 && \
                q_abs(q_sub(actual, expected)).frac <= (tolerance))

// Test Q16.16 less than
#define TEST_ASSERT_Q_LT(actual, expected) \
    TEST_ASSERT(q_lt(actual, expected))

// Test Q16.16 greater than
#define TEST_ASSERT_Q_GT(actual, expected) \
    TEST_ASSERT(q_gt(actual, expected))

#endif // QFIXED_H

// Test completion helpers
#define TEST_COMPLETE() putchar('\n')

// Test section markers (for debugging)
#define TEST_SECTION(name) /* Currently no-op */

#endif // TEST_ASSERT_H