// Debug test for pointer arithmetic issue
void putchar(int c);

int main() {
    // Test basic pointer arithmetic
    int data = 0x4142;  // 'AB'
    char* cptr = (char*)&data;
    
    // Test 1: Direct access
    putchar('1');
    putchar(':');
    putchar(' ');
    char first = *cptr;
    putchar(first);  // Should print 'B' (0x42)
    putchar('\n');
    
    // Test 2: Pointer arithmetic with intermediate variable
    putchar('2');
    putchar(':');
    putchar(' ');
    char* cptr2 = cptr + 1;
    char second = *cptr2;
    putchar(second);  // Should print 'A' (0x41)
    putchar('\n');
    
    // Test 3: Direct expression
    putchar('3');
    putchar(':');
    putchar(' ');
    putchar(*(cptr + 1));  // Should print 'A' (0x41)
    putchar('\n');
    
    // Test 4: Array-style access
    putchar('4');
    putchar(':');
    putchar(' ');
    putchar(cptr[0]);  // Should print 'B'
    putchar(cptr[1]);  // Should print 'A'
    putchar('\n');
    
    // Test 5: Check addresses are different
    putchar('5');
    putchar(':');
    putchar(' ');
    if (cptr != cptr2) {
        putchar('D');  // Different
    } else {
        putchar('S');  // Same
    }
    putchar('\n');
    
    // Test 6: Use integer for indexing
    putchar('6');
    putchar(':');
    putchar(' ');
    int i = 0;
    putchar(cptr[i]);  // Should print 'B'
    i = 1;
    putchar(cptr[i]);  // Should print 'A'
    putchar('\n');
    
    return 0;
}