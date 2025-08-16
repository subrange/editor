#ifndef ASSERT_H
#define ASSERT_H

// Assertion macro
// Note: This is a simple implementation that doesn't provide
// file/line information yet

#ifdef NDEBUG
    #define assert(expr) ((void)0)
#else
    // For now, just output 'A' for assertion failure and halt
    #define assert(expr) \
        do { \
            if (!(expr)) { \
                putchar('A'); \
                putchar('S'); \
                putchar('S'); \
                putchar('E'); \
                putchar('R'); \
                putchar('T'); \
                putchar('!'); \
                putchar('\n'); \
                /* TODO: Add proper abort/exit */ \
                while(1); /* Infinite loop for now */ \
            } \
        } while(0)
#endif

// Static assertion (compile-time)
#define static_assert(expr, msg) _Static_assert(expr, msg)

#endif // ASSERT_H