// Runtime implementation of puts
// Outputs a string followed by newline

extern void putchar(int c);

int puts(char *s) {
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