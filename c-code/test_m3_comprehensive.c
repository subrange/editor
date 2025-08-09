// M3 Comprehensive Test
// Tests: arrays, strings, pointers, global variables

void putchar(int c);

// Global variable
int global_counter = 5;

int main() {
    // Test 1: Local arrays
    int arr[3];
    arr[0] = 77;  // 'M'
    arr[1] = 51;  // '3'
    arr[2] = 58;  // ':'
    
    putchar(arr[0]);
    putchar(arr[1]);
    putchar(arr[2]);
    putchar(32);  // space
    
    // Test 2: String literals
    char *msg = "OK!";
    putchar(msg[0]);
    putchar(msg[1]);
    putchar(msg[2]);
    putchar(10);  // newline
    
    // Test 3: Pointers and address-of
    int value = 65;  // 'A'
    int *ptr = &value;
    putchar(*ptr);
    
    // Test 4: Pointer arithmetic with arrays
    int nums[2];
    nums[0] = 66;  // 'B'
    nums[1] = 67;  // 'C'
    int *p = nums;
    putchar(*p);
    putchar(*(p + 1));
    putchar(10);  // newline
    
    // Test 5: Global variable access
    if (global_counter == 5) {
        putchar(71);  // 'G'
        putchar(111); // 'o'
        putchar(111); // 'o'
        putchar(100); // 'd'
        putchar(33);  // '!'
        putchar(10);  // newline
    }
    
    return 0;
}