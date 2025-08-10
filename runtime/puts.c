// Runtime implementation of puts
// Outputs a string followed by newline
// Updated for fat pointers: pointer is passed as (address, bank_tag) pair

extern void putchar(int c);

int puts(char *s, int s_bank) {
    // The compiler will pass s as a fat pointer (2 values)
    // s_bank is ignored here since dereferencing
    // will use the correct bank based on the pointer's provenance
    
    if (!s) return -1;
    
    // Output each character
    while (*s) {
        putchar(*s);
        s = s + 1;
    }
    
    // Output newline
    putchar('\n');
    
    return 0;  // Success
}