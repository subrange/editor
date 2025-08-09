void putchar(int c);

int main() {
    // Declare an array of 5 integers
    int numbers[5];
    
    // Initialize array elements
    numbers[0] = 72;  // 'H'
    numbers[1] = 105; // 'i'
    numbers[2] = 33;  // '!'
    numbers[3] = 10;  // '\n'
    numbers[4] = 0;   // null terminator
    
    // Print array elements
    putchar(numbers[0]);
    putchar(numbers[1]);
    putchar(numbers[2]);
    putchar(numbers[3]);
    
    return 0;
}