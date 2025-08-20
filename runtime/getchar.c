// Runtime implementation of getchar
// Uses MMIO to read from TTY input

#include <mmio_constants.h>

// External MMIO functions
unsigned short mmio_read(unsigned short addr);

int getchar(void) {
    unsigned short status;
    
    // Wait until input is available
    do {
        status = mmio_read(MMIO_TTY_IN_STATUS);
    } while ((status & TTY_STATUS_HAS_BYTE) == 0);
    
    // Read and return the character
    return (int)mmio_read(MMIO_TTY_IN_POP);
}