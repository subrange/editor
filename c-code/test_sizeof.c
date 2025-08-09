// Test sizeof operator
void putchar(int c);

void print_number(int n) {
    if (n >= 10) {
        print_number(n / 10);
    }
    putchar('0' + (n % 10));
}

int main() {
    // Test sizeof on types
    int size_char = sizeof(char);
    int size_int = sizeof(int);
    int size_long = sizeof(long);
    int size_ptr = sizeof(int*);
    
    // Test sizeof on expressions
    int x = 42;
    int arr[10];
    int size_var = sizeof(x);
    int size_arr = sizeof(arr);
    
    // Print sizes
    putchar('c'); putchar('h'); putchar('a'); putchar('r'); putchar(':'); putchar(' ');
    print_number(size_char);
    putchar(10);
    
    putchar('i'); putchar('n'); putchar('t'); putchar(':'); putchar(' ');
    print_number(size_int);
    putchar(10);
    
    putchar('l'); putchar('o'); putchar('n'); putchar('g'); putchar(':'); putchar(' ');
    print_number(size_long);
    putchar(10);
    
    putchar('p'); putchar('t'); putchar('r'); putchar(':'); putchar(' ');
    print_number(size_ptr);
    putchar(10);
    
    putchar('v'); putchar('a'); putchar('r'); putchar(':'); putchar(' ');
    print_number(size_var);
    putchar(10);
    
    putchar('a'); putchar('r'); putchar('r'); putchar('['); putchar('1'); putchar('0'); putchar(']'); putchar(':'); putchar(' ');
    print_number(size_arr);
    putchar(10);
    
    return 0;
}