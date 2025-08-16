// Test NULL pointer casting between types
void putchar(int c);

int main() {
    // Test 1: Cast between different pointer types
    int* ip = (int*)0;
    char* cp = (char*)ip;
    void* vp = (void*)cp;
    long* lp = (long*)vp;
    
    if (ip == cp && cp == vp && vp == lp) {
        putchar('1');  // All NULL pointers equal regardless of type
    } else {
        putchar('N');
    }
    
    // Test 2: NULL to integer and back
    int* p1 = (int*)0;
    int addr = (int)p1;
    int* p2 = (int*)addr;
    
    if (p1 == p2 && addr == 0) {
        putchar('2');  // Round trip preserves NULL
    } else {
        putchar('N');
    }
    
    // Test 3: Non-zero integer to pointer (not NULL)
    int* p3 = (int*)100;
    if (p3 != (int*)0) {
        putchar('3');  // Non-zero address is not NULL
    } else {
        putchar('N');
    }
    
    // Test 4: Expression evaluating to 0 (not literal 0)
    int zero = 0;
    int* p4 = (int*)zero;  // Runtime 0, not compile-time NULL
    if (p4 == 0) {
        putchar('4');  // Still equals 0
    } else {
        putchar('N');
    }
    
    // Test 5: Void pointer NULL
    void* vp2 = (void*)0;
    int* ip2 = (int*)vp2;
    if (ip2 == 0) {
        putchar('5');  // void* NULL converts to typed NULL
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}