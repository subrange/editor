// Test NULL pointer dereference - should fail at runtime
void putchar(int c);

int main() {
    int* p = (int*)0;
    
    // This should cause a runtime error (NULL pointer dereference)
    int value = *p;
    
    // Should never reach here
    putchar('N');
    putchar('\n');
    return value;
}