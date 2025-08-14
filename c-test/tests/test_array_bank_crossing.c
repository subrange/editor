// Test array indexing with bank crossing via GEP
// Banks are 4096 bytes, so an int array of 2000 elements (8000 bytes) spans 2 banks
void putchar(int c);

int main() {
    int huge_array[2000];  // 8000 bytes, spans 2 banks!
    
    // Initialize some values
    huge_array[0] = 42;      // Bank 0
    huge_array[1000] = 100;  // Still in Bank 0 (4000 bytes)
    huge_array[1500] = 99;   // Bank 1 (6000 bytes)
    huge_array[1999] = 77;   // Bank 1 (7996 bytes)
    
    // Test reading from Bank 0
    if (huge_array[0] == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test reading from still Bank 0
    if (huge_array[1000] == 100) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test reading from Bank 1 (crosses bank boundary)
    if (huge_array[1500] == 99) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test reading from end of Bank 1
    if (huge_array[1999] == 77) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test pointer arithmetic across banks
    int *p = &huge_array[0];
    int *q = p + 1500;  // Should point to huge_array[1500] in Bank 1
    if (*q == 99) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}