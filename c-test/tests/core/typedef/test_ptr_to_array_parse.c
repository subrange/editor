// Test parsing of pointer to array
void putchar(int c);

int main() {
    // Test different declarations
    int (*p1)[3];     // pointer to array of 3 ints
    int *p2[3];       // array of 3 pointers to int
    
    putchar('Y');
    return 0;
}