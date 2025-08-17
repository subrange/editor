// Test that function declarations and definitions are handled properly
// This tests the fix for the "Redefinition of symbol" error

// Function declaration (prototype)
int add(int a, int b);

// Another declaration of the same function (should be allowed)
int add(int a, int b);

// External function declaration
extern int multiply(int x, int y);

// Function definition (should be allowed after declaration)
int add(int a, int b) {
    return a + b;
}

// Another external declaration (should be allowed)
extern int multiply(int x, int y);

// Function with no prior declaration
int subtract(int a, int b) {
    return a - b;
}

// Definition of external function
int multiply(int x, int y) {
    return x * y;
}

int main() {
    int result = add(5, 3);
    if (result == 8) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    result = multiply(4, 3);
    if (result == 12) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    result = subtract(10, 3);
    if (result == 7) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}