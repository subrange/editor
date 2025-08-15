// Test basic NULL pointer operations
void putchar(int c);

int main() {
    // Test 1: NULL pointer creation
    int* p1 = (int*)0;
    void* p2 = (void*)0;
    char* p3 = (char*)0;
    
    // Test 2: NULL pointer comparison with 0
    if (p1 == 0) {
        putchar('1');  // Should print
    } else {
        putchar('N');
    }
    
    if (0 == p2) {
        putchar('2');  // Should print
    } else {
        putchar('N');
    }
    
    // Test 3: NULL pointers are equal to each other
    if (p1 == p2) {
        putchar('3');  // Should print
    } else {
        putchar('N');
    }
    
    if (p2 == p3) {
        putchar('4');  // Should print
    } else {
        putchar('N');
    }
    
    // Test 4: NULL pointer in conditionals
    if (!p1) {
        putchar('5');  // Should print (NULL is falsy)
    } else {
        putchar('N');
    }
    
    if (p2) {
        putchar('N');
    } else {
        putchar('6');  // Should print (NULL is falsy)
    }
    
    putchar('\n');
    return 0;
}