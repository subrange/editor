// Test NULL pointer arithmetic
void putchar(int c);

int main() {
    int* p = (int*)0;
    
    // Test 1: NULL + offset
    int* q = p + 5;
    if (q == (int*)5) {  // NULL + 5 should equal address 5
        putchar('1');  // Should print
    } else {
        putchar('N');
    }
    
    // Test 2: NULL - offset (for pointers that support it)
    int* r = q - 3;
    if (r == (int*)2) {  // 5 - 3 = 2
        putchar('2');  // Should print
    } else {
        putchar('N');
    }
    
    // Test 3: Pointer difference with NULL
    int* s = (int*)10;
    int diff = s - p;  // 10 - 0 = 10
    if (diff == 10) {
        putchar('3');  // Should print
    } else {
        putchar('N');
    }
    
    // Test 4: NULL pointer increment
    int* t = (int*)0;
    t++;  // Should become (int*)1
    if (t == (int*)1) {
        putchar('4');  // Should print
    } else {
        putchar('N');
    }
    
    // Test 5: Array indexing on NULL (syntactically valid, runtime error if dereferenced)
    int* u = (int*)0;
    int* v = &u[10];  // Same as u + 10
    if (v == (int*)10) {
        putchar('5');  // Should print
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}