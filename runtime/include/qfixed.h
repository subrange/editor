#ifndef QFIXED_H
#define QFIXED_H

// Q16.16 Fixed-Point Library for Ripple VM
// 16 bits integer part, 16 bits fractional part
// Range: -32768.0 to 32767.99998
// Precision: 1/65536 ≈ 0.0000153

// Core type definition
typedef struct {
    short integer;          // Integer part (signed)
    unsigned short frac;    // Fractional part (unsigned)
} q16_16_t;

// Constants
#define Q16_16_ONE ((q16_16_t){1, 0})
#define Q16_16_ZERO ((q16_16_t){0, 0})
#define Q16_16_HALF ((q16_16_t){0, 0x8000})
#define Q16_16_PI ((q16_16_t){3, 0x243F})      // 3.14159265
#define Q16_16_E ((q16_16_t){2, 0xB7E1})       // 2.71828183
#define Q16_16_SQRT2 ((q16_16_t){1, 0x6A09})   // 1.41421356

// Conversion macros
#define Q16_16_FROM_INT(x) ((q16_16_t){(x), 0})
#define Q16_16_TO_INT(x) ((x).integer)
#define Q16_16_FRAC_BITS 16
#define Q16_16_FRAC_MASK 0xFFFF

// Basic arithmetic operations
q16_16_t q_add(q16_16_t a, q16_16_t b);
q16_16_t q_sub(q16_16_t a, q16_16_t b);
q16_16_t q_mul(q16_16_t a, q16_16_t b);
q16_16_t q_div(q16_16_t a, q16_16_t b);
q16_16_t q_neg(q16_16_t x);
q16_16_t q_abs(q16_16_t x);

// Comparison operations
int q_eq(q16_16_t a, q16_16_t b);
int q_lt(q16_16_t a, q16_16_t b);
int q_le(q16_16_t a, q16_16_t b);
int q_gt(q16_16_t a, q16_16_t b);
int q_ge(q16_16_t a, q16_16_t b);

// Conversion functions
q16_16_t q_from_int(int x);
// q16_16_t q_from_float(float x);  // Not supported yet - no float type
// q16_16_t q_from_double(double x); // Not supported yet - no double type
int q_to_int(q16_16_t x);
int q_to_int_round(q16_16_t x);
// float q_to_float(q16_16_t x);     // Not supported yet - no float type
// double q_to_double(q16_16_t x);   // Not supported yet - no double type

// String conversion
q16_16_t q_from_string(const char *str);
void q_to_string(q16_16_t x, char *buf, int precision);

// Advanced math functions
q16_16_t q_sqrt(q16_16_t x);
q16_16_t q_reciprocal(q16_16_t x);
q16_16_t q_sin(q16_16_t x);
q16_16_t q_cos(q16_16_t x);
q16_16_t q_tan(q16_16_t x);
q16_16_t q_exp(q16_16_t x);
q16_16_t q_log(q16_16_t x);
q16_16_t q_pow(q16_16_t base, q16_16_t exp);

// Utility functions
q16_16_t q_floor(q16_16_t x);
q16_16_t q_ceil(q16_16_t x);
q16_16_t q_round(q16_16_t x);
q16_16_t q_min(q16_16_t a, q16_16_t b);
q16_16_t q_max(q16_16_t a, q16_16_t b);
q16_16_t q_clamp(q16_16_t x, q16_16_t min, q16_16_t max);

// Linear interpolation
q16_16_t q_lerp(q16_16_t a, q16_16_t b, q16_16_t t);

// Macro implementations for performance-critical operations (removed to avoid conflicts)

#endif // QFIXED_H