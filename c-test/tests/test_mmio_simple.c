// Simple test of new MMIO functionality using runtime functions

#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

int main() {
    // Test basic TTY output using stdio putchar
    putchar('H');
    putchar('i');
    putchar('\n');
    
    // Test RNG using mmio functions
    unsigned short rng1 = rng_get();
    unsigned short rng2 = rng_get();
    
    // RNG values should be different
    if (rng1 != rng2) {
        putchar('O');
        putchar('K');
    } else {
        putchar('N');
        putchar('O');
    }
    putchar('\n');
    
    // Additional test: verify direct MMIO access works
    // Test writing and reading RNG seed
    rng_set_seed(0x5678);
    unsigned short seed = rng_get_seed();
    
    if (seed == 0x5678) {
        puts("SEED:OK");
    } else {
        puts("SEED:NO");
    }
    
    // Test using mmio_read/write directly for TTY
    // Check TTY status
    unsigned short tty_status = mmio_read(MMIO_TTY_STATUS);
    if (tty_status & TTY_STATUS_READY) {
        // TTY is ready, output a test character
        mmio_write(MMIO_TTY_OUT, '!');
        mmio_write(MMIO_TTY_OUT, '\n');
    }
    
    return 0;
}