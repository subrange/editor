// Test pointer cast expressions (no integer casts yet)
void putchar(int c);

int main() {
    // Pointer to pointer casts
    int x = 42;
    int* ptr = &x;
    void* vptr = (void*)ptr;  // Cast int* to void*
    int* ptr2 = (int*)vptr;   // Cast void* to int*
    
    if (*ptr2 == 42) {
        putchar('Y');  // Success
    } else {
        putchar('N');  // Failure
    }
    
    putchar('\n');
    return 0;
}