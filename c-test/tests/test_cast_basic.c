// Test basic cast expressions
void putchar(int c);

int main() {
    // Integer to integer casts
    int x = 42;
    char c = (char)x;  // Cast int to char
    
    if (c == 42) {
        putchar('Y');  // Success
    } else {
        putchar('N');  // Failure
    }
    
    // Pointer casts
    int y = 100;
    int* ptr = &y;
    void* vptr = (void*)ptr;  // Cast int* to void*
    int* ptr2 = (int*)vptr;   // Cast void* to int*
    
    if (*ptr2 == 100) {
        putchar('Y');  // Success
    } else {
        putchar('N');  // Failure
    }
    
    // Integer to pointer cast (NULL)
    void* null_ptr = (void*)0;
    
    if (null_ptr == 0) {
        putchar('Y');  // Success
    } else {
        putchar('N');  // Failure
    }
    
    putchar('\n');
    return 0;
}