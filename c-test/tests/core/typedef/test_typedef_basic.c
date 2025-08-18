void putchar(int c);

// Basic typedef tests
typedef int integer;
typedef char character;
typedef int* int_ptr;

int main() {
    // Test basic typedef
    integer i = 65;
    putchar(i); // 'A'
    
    // Test char typedef
    character c = 'B';
    putchar(c); // 'B'
    
    // Test pointer typedef
    integer value = 67;
    int_ptr ptr = &value;
    putchar(*ptr); // 'C'
    
    // Test typedef in expressions
    integer a = 30;
    integer b = 38;
    integer sum = a + b;
    putchar(sum); // 'D' (68)
    
    putchar('\n');
    return 0;
}