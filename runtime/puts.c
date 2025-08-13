// Runtime implementation of puts
// Outputs a string followed by newline
// The compiler automatically handles fat pointers

extern void putchar(int c);

int puts(char *s) {
    // The compiler will pass s as a fat pointer (address + bank tag)
    // and handle the bank tag automatically for all operations
    
//     if (!s) return -1;
    
    // Output each character
    while (*s) {
        putchar(*s);
        s = s + 1;
    }
    
    // Output newline
    putchar('\n');
    
    return 0;  // Success
}