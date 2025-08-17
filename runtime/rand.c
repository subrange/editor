// Standard C library rand() and srand() implementation
// Uses the Ripple VM's hardware RNG for better randomness

#include <mmio.h>

// Static variable to track if srand() has been called
static int rand_initialized = 0;

// Set the random seed
void srand(unsigned int seed) {
    // Set the RNG seed using MMIO
    // We only use the low 16 bits for now
    unsigned short seed16 = (unsigned short)(seed & 0xFFFF);
    
    // Use the mmio function to set the seed
    rng_set_seed(seed16);
    
    rand_initialized = 1;
}

// Get a random number in the range [0, RAND_MAX]
// RAND_MAX is typically 32767 (0x7FFF) for 16-bit systems
int rand(void) {
    // If srand() hasn't been called, use a default seed
    if (!rand_initialized) {
        // Use a simple default seed
        srand(1);
    }
    
    // Read from the hardware RNG using the mmio function
    unsigned short result = rng_get();
    
    // Mask to ensure it's in the range [0, 32767]
    // This matches the standard RAND_MAX for many C implementations
    return (int)(result & 0x7FFF);
}