// Runtime implementation of memcpy
// Copies n bytes from src to dest
// Updated for fat pointers: pointers are passed as (address, bank_tag) pairs

// Return type changed to void to avoid ABI complications with fat pointers
void memcpy(char *dest, int dest_bank, char *src, int src_bank, int n) {
    // The compiler will pass dest and src as fat pointers (2 values each)
    // dest_bank and src_bank are ignored here since array indexing
    // will use the correct bank based on the pointer's provenance
    
    for (int i = 0; i < n; i = i + 1) {
        dest[i] = src[i];
    }
    
    // No return value to avoid fat pointer ABI issues
}