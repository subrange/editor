// malloc.c - Simple bump allocator for heap memory
// Each bank is treated as a separate allocation pool
// Allocations never span banks - if it doesn't fit, we move to the next bank

#include <stddef.h>

// Heap configuration
#define HEAP_START_BANK 3      // Heap starts at bank 3 (after code, globals, stack)
#define HEAP_END_BANK 255      // Maximum bank we can use for heap
#define BANK_SIZE 65535         // Size of each bank in words (16-bit) - match compiler default

// Heap state - stored in globals
static unsigned int current_heap_bank = HEAP_START_BANK;
static unsigned int current_heap_offset = 0;
static unsigned char heap_initialized = 0;

// Initialize heap if not already done
static void init_heap(void) {
    if (!heap_initialized) {
        current_heap_bank = HEAP_START_BANK;
        current_heap_offset = 0;
        heap_initialized = 1;
    }
}

// Helper function to construct a fat pointer from address and bank
// Uses inline assembly to properly set both components
static void* make_fat_ptr(unsigned int addr, unsigned int bank) {
    void* result;
    // Use inline assembly to properly return a fat pointer
    // RV0 = address, RV1 = bank for fat pointer returns
    __asm__(
        "move rv0, %1\n"     // Move address to RV0
        "move rv1, %2\n"     // Move bank to RV1
        : "=r"(result)       // Output (dummy, real result is in RV0/RV1)
        : "r"(addr), "r"(bank)
        : "rv0", "rv1"
    );
    return result;
}

// Simple bump allocator
void* malloc(int size) {
    init_heap();
    
    // Validate size
    if (size <= 0) {
        return NULL;
    }
    
    // Check if allocation would exceed bank size
    if (size > BANK_SIZE) {
        // Allocation too large for a single bank - error
        return NULL;
    }
    
    // Check if allocation fits in current bank
    if (current_heap_offset + size > BANK_SIZE) {
        // Doesn't fit - move to next bank
        current_heap_bank++;
        current_heap_offset = 0;
        
        // Check if we've exhausted heap banks
        if (current_heap_bank > HEAP_END_BANK) {
            // Out of memory
            return NULL;
        }
    }
    
    // Save the allocation position
    unsigned int alloc_offset = current_heap_offset;
    unsigned int alloc_bank = current_heap_bank;
    
    // Bump the allocation pointer
    current_heap_offset += size;
    
    // Return the fat pointer with address and bank
    return make_fat_ptr(alloc_offset, alloc_bank);
}

// Free is a no-op for bump allocator
// We keep the function for API compatibility
void free(void* ptr) {
    // Bump allocator doesn't support freeing individual allocations
    // In a real implementation, we might track allocations for debugging
    (void)ptr; // Suppress unused parameter warning
}

// Calloc - allocate and zero memory
void* calloc(int nmemb, int size) {
    // Calculate total size, checking for overflow
    int total = nmemb * size;
    if (nmemb != 0 && total / nmemb != size) {
        // Overflow detected
        return NULL;
    }
    
    void* ptr = malloc(total);
    if (ptr != NULL) {
        // Zero the allocated memory
        // We'll need memset for this
        unsigned int bank = ((unsigned int)ptr) >> 16;
        unsigned int offset = ((unsigned int)ptr) & 0xFFFF;
        
        // Manual zeroing since we don't have memset yet
        // This will be replaced with memset once available
        for (int i = 0; i < total; i++) {
            // Store zero at each location
            // This will need proper assembly implementation
            *((char*)ptr + i) = 0;
        }
    }
    
    return ptr;
}

// Realloc - not supported in bump allocator
void* realloc(void* ptr, int size) {
    // For bump allocator, we can't resize existing allocations
    // We could allocate new space and copy, but that's inefficient
    if (ptr == NULL) {
        return malloc(size);
    }
    
    // Can't resize existing allocation in bump allocator
    return NULL;
}