// Direct test of MMIO without function calls

void putchar(int c);

int main() {
    // Direct output to TTY_OUT at bank 0, word 0
    // Using inline assembly-like approach with pointer arithmetic
    unsigned short* mmio = (unsigned short*)0;
    
    // Output 'A' directly
    mmio[0] = 65;  // 'A'
    mmio[0] = 10;  // '\n'
    
    // Read from RNG at word 4
    unsigned short rng_val = mmio[4];
    
    // Output based on RNG value
    if (rng_val != 0) {
        mmio[0] = 89;  // 'Y'
    } else {
        mmio[0] = 78;  // 'N'
    }
    mmio[0] = 10;  // '\n'
    
    return 0;
}