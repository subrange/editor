// Test case isolated from T10 of test_s3_kitchen_sink.c
// Testing pointer casting between different types

void putchar(int c);

void my_puts(char* str);

int main() {
    my_puts("Testing pointer casts:");
    
    // Initialize data with known value
    int data = 0x4142;  // 'AB' in little endian (0x42='B', 0x41='A')
    
    // Test 1: Basic cast through void*
    void* vptr = (void*)&data;
    int* iptr = (int*)vptr;
    
    // Check if int* cast works
    if (*iptr == 0x4142) {
        putchar('Y');  // Should print 'Y'
    } else {
        putchar('N');
    }
    putchar('\n');
    
    // Test 2: Cast to char* and access bytes
    char* cptr = (char*)vptr;
    
    // Print the raw value we're going to access
    my_puts("First byte (should be 0x42='B'):");
    putchar(*cptr);  // First byte - should be 'B' (0x42)
    putchar('\n');
    
    my_puts("Second byte (should be 0x41='A'):");
    putchar(*(cptr + 1));  // Second byte - should be 'A' (0x41)
    putchar('\n');
    
    // Test 3: Try with a simpler value
    my_puts("Simple test with known chars:");
    int simple = 0x4344;  // 'CD'
    char* sptr = (char*)&simple;
    putchar(*sptr);  // Should print 'D' (0x44)
    putchar(*(sptr + 1));  // Should print 'C' (0x43)
    putchar('\n');
    
    // Test 4: Direct char array for comparison
    my_puts("Direct char array:");
    char direct[2] = {'E', 'F'};
    putchar(direct[0]);  // Should print 'E'
    putchar(direct[1]);  // Should print 'F'
    putchar('\n');
    
    return 0;
}