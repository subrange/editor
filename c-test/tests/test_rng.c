// Test RNG functionality
#include <mmio.h>
#include <stdio.h>

void print_hex_digit(unsigned int n) {
    n = n & 0xF;  // Only keep bottom 4 bits
    if (n < 10) {
        putchar('0' + n);
    } else {
        putchar('A' + (n - 10));
    }
}

void print_hex16(unsigned short value) {
    // Print 16-bit value as 4 hex digits using bit shifts
    print_hex_digit(value >> 12);   // Top 4 bits
    print_hex_digit(value >> 8);    // Next 4 bits  
    print_hex_digit(value >> 4);    // Next 4 bits
    print_hex_digit(value);          // Bottom 4 bits
}

int main() {
    // Test that we can output through putchar
    putchar('R');
    putchar('N');
    putchar('G');
    putchar(':');
    putchar(' ');

    // Get the 16-bit random value
    unsigned short rng = rng_get();
    
    // Print as hex to see the full 16-bit value
    putchar('0');
    putchar('x');
    print_hex16(rng);
    
    putchar('\n');
    
    // Also let's print the decimal value
    putchar('D');
    putchar('E');
    putchar('C');
    putchar(':');
    putchar(' ');
    
    // Print decimal value (up to 65535)
    unsigned short val = rng;
    if (val >= 10000) {
        putchar('0' + (val / 10000));
        val = val % 10000;
    }
    if (rng >= 1000 || val != rng) {  // Print if we already printed a digit
        putchar('0' + (val / 1000));
        val = val % 1000;
    }
    if (rng >= 100 || val != rng) {
        putchar('0' + (val / 100));
        val = val % 100;
    }
    if (rng >= 10 || val != rng) {
        putchar('0' + (val / 10));
        val = val % 10;
    }
    putchar('0' + val);
    
    putchar('\n');
    
    return 0;
}
