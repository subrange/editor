// Test new MMIO layout with TTY, RNG, and TEXT40 display

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

int main() {
    // Test 1: Basic TTY output using stdio putchar
    putchar('T');
    putchar('T');
    putchar('Y');
    putchar(':');
    putchar('O');
    putchar('K');
    putchar('\n');
    
    // Test 2: Read RNG values using mmio functions
    unsigned short rng1 = rng_get();
    unsigned short rng2 = rng_get();
    unsigned short rng3 = rng_get();
    
    // RNG values should be different
    if (rng1 != rng2 && rng2 != rng3) {
        puts("RNG:OK");
    } else {
        puts("RNG:NO");
    }
    
    // Test 3: TEXT40 display mode using display functions
    display_set_mode(DISP_MODE_TEXT40);
    display_clear();
    display_enable();
    
    // Write "HELLO" to first line using text40 functions
    text40_puts(0, 0, "HELLO");
    
    // Write "WORLD" to second line
    text40_puts(0, 1, "WORLD");
    
    // Write individual characters to test text40_putchar
    text40_putchar(0, 3, 'T');
    text40_putchar(1, 3, 'E');
    text40_putchar(2, 3, 'S');
    text40_putchar(3, 3, 'T');
    
    // Flush display
    display_flush();
    
    // Check display status using mmio_read
    unsigned short status = mmio_read(MMIO_DISP_STATUS);
    if (status & DISP_STATUS_FLUSH_DONE) {
        puts("DISP:OK");
    } else {
        puts("DISP:NO");
    }
    
    // Test direct MMIO access for completeness
    // Write a test pattern to verify mmio_write works
    mmio_write(MMIO_RNG_SEED, 0x1234);
    unsigned short seed = rng_get_seed();
    if (seed == 0x1234) {
        puts("MMIO:OK");
    } else {
        puts("MMIO:NO");
    }
    
    return 0;
}