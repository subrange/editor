// Runtime implementation of memset
// Sets n bytes of memory to a given value
// Updated for fat pointers: pointer is passed as (address, bank_tag) pair

void memset(char *s, int c, int n) {
    // The compiler will pass s as a fat pointer (address + bank tag)
    // and handle the bank tag automatically for all operations
    
    for (int i = 0; i < n; i = i + 1) {
        s[i] = c;  // c will be truncated to char automatically
    }
    
    // No return value to avoid fat pointer ABI issues
}