// Test NULL pointer comparisons
void putchar(int c);

int main() {
    int* null1 = (int*)0;
    void* null2 = (void*)0;
    char* null3 = (char*)0;
    
    int x = 42;
    int* valid = &x;
    
    // Test 1: All NULL pointers are equal
    if (null1 == null2 && null2 == null3 && null1 == null3) {
        putchar('1');  // Should print
    } else {
        putchar('N');
    }
    
    // Test 2: NULL != valid pointer
    if (null1 != valid) {
        putchar('2');  // Should print
    } else {
        putchar('N');
    }
    
    if (valid != null2) {
        putchar('3');  // Should print
    } else {
        putchar('N');
    }
    
    // Test 3: Relational comparisons with NULL
    // NULL (0) is less than any valid pointer
    if (null1 < valid) {
        putchar('4');  // Should print
    } else {
        putchar('N');
    }
    
    if (valid > null2) {
        putchar('5');  // Should print
    } else {
        putchar('N');
    }
    
    // Test 4: NULL <= NULL and NULL >= NULL
    if (null1 <= null2 && null2 >= null3) {
        putchar('6');  // Should print
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}