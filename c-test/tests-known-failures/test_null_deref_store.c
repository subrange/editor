// Test NULL pointer store - should fail at runtime
void putchar(int c);

int main() {
    int* p = (int*)0;
    
    // This should cause a runtime error (NULL pointer dereference)
    *p = 42;
    
    // Should never reach here
    putchar('N');
    putchar('\n');
    return 0;
}