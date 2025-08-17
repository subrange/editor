// Test RNG seed functionality
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
    // Set a specific seed
    rng_set_seed(0x1234);
    
    putchar('S');
    putchar('e');
    putchar('e');
    putchar('d');
    putchar(':');
    putchar(' ');
    putchar('0');
    putchar('x');
    
    // Read back the seed to verify it was set
    unsigned short seed = rng_get_seed();
    print_hex16(seed);
    putchar('\n');
    
    // Generate a few random numbers with this seed
    putchar('R');
    putchar('1');
    putchar(':');
    putchar(' ');
    unsigned short r1 = rng_get();
    print_hex16(r1);
    putchar('\n');
    
    putchar('R');
    putchar('2');
    putchar(':');
    putchar(' ');
    unsigned short r2 = rng_get();
    print_hex16(r2);
    putchar('\n');
    
    // Set a different seed
    rng_set_seed(0x5678);
    
    putchar('N');
    putchar('e');
    putchar('w');
    putchar(' ');
    putchar('s');
    putchar('e');
    putchar('e');
    putchar('d');
    putchar(':');
    putchar(' ');
    putchar('0');
    putchar('x');
    
    seed = rng_get_seed();
    print_hex16(seed);
    putchar('\n');
    
    // Generate a random number with the new seed
    putchar('R');
    putchar('3');
    putchar(':');
    putchar(' ');
    unsigned short r3 = rng_get();
    print_hex16(r3);
    putchar('\n');
    
    return 0;
}