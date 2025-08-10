void putchar(int c);

int main() {
    // Declare an array of 5 integers
    int numbers[5];
    
    // Initialize array elements
    numbers[0] = 72;  // 'H'
    numbers[1] = 105; // 'i'
    numbers[2] = 33;  // '!'
    numbers[3] = 10;  // '\n'
    numbers[4] = 99;  // test value
    
    // Test array indexing works correctly
    if (numbers[0] == 72) {
        putchar('1');
    } else {
        putchar('X');
    }
    
    if (numbers[1] == 105) {
        putchar('2');
    } else {
        putchar('X');
    }
    
    if (numbers[4] == 99) {
        putchar('3');
    } else {
        putchar('X');
    }
    
    putchar('\n');
    
    return 0;
}