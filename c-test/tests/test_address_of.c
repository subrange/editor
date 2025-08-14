// Test address-of and dereference operators

int puts(char* c);

int main() {
    int x = 42;
    int *ptr = &x;  // Take address of x
    
    // Modify through pointer
    *ptr = 100;
    
    // x should now be 100
    if (x == 100) {
        puts("OK");
    } else {
        puts("Failed to modify x through pointer");
    }
    
    return 0;
}