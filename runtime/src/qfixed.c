// Q16.16 Fixed-Point Implementation for Ripple VM
// Optimized for 16-bit architecture

#include "qfixed.h"

// Helper function to combine integer and fractional parts into 32-bit value
static unsigned int q_to_raw(q16_16_t x) {
    return ((unsigned int)(x.integer & 0xFFFF) << 16) | x.frac;
}

// Helper function to split 32-bit value into Q16.16
static q16_16_t q_from_raw(unsigned int raw) {
    q16_16_t result;
    result.integer = (short)(raw >> 16);
    result.frac = (unsigned short)(raw & 0xFFFF);
    return result;
}

// Basic arithmetic operations

q16_16_t q_add(q16_16_t a, q16_16_t b) {
    unsigned int frac_sum = a.frac + b.frac;
    q16_16_t result;
    result.frac = frac_sum & 0xFFFF;
    result.integer = a.integer + b.integer + (frac_sum >> 16);
    return result;
}

q16_16_t q_sub(q16_16_t a, q16_16_t b) {
    q16_16_t result;
    if (a.frac >= b.frac) {
        result.frac = a.frac - b.frac;
        result.integer = a.integer - b.integer;
    } else {
        result.frac = (0x10000 + a.frac) - b.frac;
        result.integer = a.integer - b.integer - 1;
    }
    return result;
}

q16_16_t q_mul(q16_16_t a, q16_16_t b) {
    // Break down multiplication into partial products
    // (a_int.a_frac) * (b_int.b_frac)
    
    // Get absolute values and track sign
    int sign = 0;
    if (a.integer < 0) {
        sign = sign ^ 1;
        a = q_neg(a);
    }
    if (b.integer < 0) {
        sign = sign ^ 1;
        b = q_neg(b);
    }
    
    // Partial products (each fits in 32 bits)
    unsigned int int_int = (unsigned short)a.integer * (unsigned short)b.integer;
    unsigned int int_frac_a = (unsigned short)a.integer * b.frac;
    unsigned int int_frac_b = (unsigned short)b.integer * a.frac;
    unsigned int frac_frac = ((unsigned int)a.frac * b.frac) >> 16;
    
    // Combine partial products
    unsigned int result_int = int_int;
    unsigned int result_frac = int_frac_a + int_frac_b + frac_frac;
    
    // Handle carry from fractional to integer part
    result_int = result_int + (result_frac >> 16);
    result_frac = result_frac & 0xFFFF;
    
    q16_16_t result;
    result.integer = (short)result_int;
    result.frac = (unsigned short)result_frac;
    
    // Apply sign
    if (sign) {
        result = q_neg(result);
    }
    
    return result;
}

q16_16_t q_div(q16_16_t a, q16_16_t b) {
    // Check for division by zero
    if (b.integer == 0 && b.frac == 0) {
        // Return max value as error indicator
        return (q16_16_t){0x7FFF, 0xFFFF};
    }
    
    // Get absolute values and track sign
    int sign = 0;
    if (a.integer < 0) {
        sign = sign ^ 1;
        a = q_neg(a);
    }
    if (b.integer < 0) {
        sign = sign ^ 1;
        b = q_neg(b);
    }
    
    // Convert to raw 32-bit values for division
    unsigned int dividend = q_to_raw(a);
    unsigned int divisor = q_to_raw(b);
    
    // Shift dividend left by 16 bits for fixed-point division
    // This is equivalent to multiplying by 2^16
    unsigned int quotient_high = dividend / divisor;
    unsigned int remainder = dividend % divisor;
    
    // Calculate additional precision
    unsigned int quotient_low = (remainder << 16) / divisor;
    
    // Combine results
    unsigned int result = (quotient_high << 16) | quotient_low;
    
    q16_16_t q_result = q_from_raw(result);
    
    // Apply sign
    if (sign) {
        q_result = q_neg(q_result);
    }
    
    return q_result;
}

q16_16_t q_neg(q16_16_t x) {
    // Two's complement negation
    unsigned int raw = q_to_raw(x);
    raw = (~raw) + 1;
    return q_from_raw(raw);
}

q16_16_t q_abs(q16_16_t x) {
    if (x.integer < 0) {
        return q_neg(x);
    }
    return x;
}

// Comparison operations

int q_eq(q16_16_t a, q16_16_t b) {
    return (a.integer == b.integer) && (a.frac == b.frac);
}

int q_lt(q16_16_t a, q16_16_t b) {
    if (a.integer < b.integer) return 1;
    if (a.integer > b.integer) return 0;
    return a.frac < b.frac;
}

int q_le(q16_16_t a, q16_16_t b) {
    return q_lt(a, b) || q_eq(a, b);
}

int q_gt(q16_16_t a, q16_16_t b) {
    return q_lt(b, a);
}

int q_ge(q16_16_t a, q16_16_t b) {
    return !q_lt(a, b);
}

// Conversion functions

q16_16_t q_from_int(int x) {
    q16_16_t result;
    result.integer = (short)x;
    result.frac = 0;
    return result;
}

int q_to_int(q16_16_t x) {
    return x.integer;
}

int q_to_int_round(q16_16_t x) {
    if (x.frac >= 0x8000) {
        return x.integer + 1;
    }
    return x.integer;
}

// Square root using Newton-Raphson method
q16_16_t q_sqrt(q16_16_t x) {
    // Check for negative input
    if (x.integer < 0) {
        // Return 0 as error indicator
        return Q16_16_ZERO;
    }
    
    // Special case for zero
    if (x.integer == 0 && x.frac == 0) {
        return Q16_16_ZERO;
    }
    
    // Initial guess: x/2 for x >= 1, otherwise 0.5
    q16_16_t guess;
    if (x.integer >= 1) {
        guess.integer = x.integer >> 1;
        guess.frac = x.frac >> 1;
        if (x.integer & 1) {
            guess.frac = guess.frac | 0x8000;
        }
    } else {
        guess = Q16_16_HALF;
    }
    
    // Newton-Raphson iterations
    // Next = (guess + x/guess) / 2
    for (int i = 0; i < 8; i++) {
        q16_16_t x_over_guess = q_div(x, guess);
        q16_16_t sum = q_add(guess, x_over_guess);
        
        // Divide by 2 (shift right by 1)
        q16_16_t next;
        next.integer = sum.integer >> 1;
        next.frac = sum.frac >> 1;
        if (sum.integer & 1) {
            next.frac = next.frac | 0x8000;
        }
        
        // Check for convergence
        if (q_eq(next, guess)) {
            break;
        }
        
        guess = next;
    }
    
    return guess;
}

// Reciprocal (1/x) using Newton-Raphson
q16_16_t q_reciprocal(q16_16_t x) {
    // Check for zero
    if (x.integer == 0 && x.frac == 0) {
        // Return max value as error indicator
        return (q16_16_t){0x7FFF, 0xFFFF};
    }
    
    // Initial guess based on magnitude
    q16_16_t guess;
    if (q_abs(x).integer >= 2) {
        // For |x| >= 2, start with 0.5
        guess = Q16_16_HALF;
    } else if (q_abs(x).integer >= 1) {
        // For 1 <= |x| < 2, start with 1
        guess = Q16_16_ONE;
    } else {
        // For |x| < 1, start with 2
        guess = (q16_16_t){2, 0};
    }
    
    // Newton-Raphson iterations: guess = guess * (2 - x * guess)
    q16_16_t two = (q16_16_t){2, 0};
    
    for (int i = 0; i < 8; i++) {
        q16_16_t x_guess = q_mul(x, guess);
        q16_16_t two_minus = q_sub(two, x_guess);
        q16_16_t next = q_mul(guess, two_minus);
        
        // Check for convergence
        if (q_eq(next, guess)) {
            break;
        }
        
        guess = next;
    }
    
    return guess;
}

// Utility functions

q16_16_t q_floor(q16_16_t x) {
    q16_16_t result;
    result.integer = x.integer;
    result.frac = 0;
    if (x.integer < 0 && x.frac > 0) {
        result.integer--;
    }
    return result;
}

q16_16_t q_ceil(q16_16_t x) {
    q16_16_t result;
    result.integer = x.integer;
    result.frac = 0;
    if (x.frac > 0) {
        result.integer++;
    }
    return result;
}

q16_16_t q_round(q16_16_t x) {
    q16_16_t result;
    result.integer = x.integer;
    result.frac = 0;
    if (x.frac >= 0x8000) {
        result.integer++;
    }
    return result;
}

q16_16_t q_min(q16_16_t a, q16_16_t b) {
    return q_lt(a, b) ? a : b;
}

q16_16_t q_max(q16_16_t a, q16_16_t b) {
    return q_gt(a, b) ? a : b;
}

q16_16_t q_clamp(q16_16_t x, q16_16_t min, q16_16_t max) {
    if (q_lt(x, min)) return min;
    if (q_gt(x, max)) return max;
    return x;
}

// Linear interpolation: a + (b - a) * t
q16_16_t q_lerp(q16_16_t a, q16_16_t b, q16_16_t t) {
    q16_16_t diff = q_sub(b, a);
    q16_16_t scaled = q_mul(diff, t);
    return q_add(a, scaled);
}

// Simple sine approximation using Taylor series
// sin(x) ≈ x - x³/6 + x⁵/120 - x⁷/5040
q16_16_t q_sin(q16_16_t x) {
    // Reduce x to range [-π, π]
    q16_16_t two_pi = (q16_16_t){6, 0x487E};  // 2π ≈ 6.28318
    
    while (q_gt(x, Q16_16_PI)) {
        x = q_sub(x, two_pi);
    }
    while (q_lt(x, q_neg(Q16_16_PI))) {
        x = q_add(x, two_pi);
    }
    
    // Taylor series
    q16_16_t x2 = q_mul(x, x);
    q16_16_t x3 = q_mul(x2, x);
    q16_16_t x5 = q_mul(x3, x2);
    q16_16_t x7 = q_mul(x5, x2);
    
    // Coefficients
    q16_16_t c3 = q_div(Q16_16_ONE, q_from_int(6));      // 1/6
    q16_16_t c5 = q_div(Q16_16_ONE, q_from_int(120));    // 1/120
    q16_16_t c7 = q_div(Q16_16_ONE, q_from_int(5040));   // 1/5040
    
    // Calculate series
    q16_16_t term1 = x;
    q16_16_t term2 = q_mul(x3, c3);
    q16_16_t term3 = q_mul(x5, c5);
    q16_16_t term4 = q_mul(x7, c7);
    
    q16_16_t result = term1;
    result = q_sub(result, term2);
    result = q_add(result, term3);
    result = q_sub(result, term4);
    
    return result;
}

// Cosine using sin(x + π/2)
q16_16_t q_cos(q16_16_t x) {
    q16_16_t pi_half = (q16_16_t){1, 0x921F};  // π/2 ≈ 1.5708
    return q_sin(q_add(x, pi_half));
}

// Tangent = sin/cos
q16_16_t q_tan(q16_16_t x) {
    return q_div(q_sin(x), q_cos(x));
}