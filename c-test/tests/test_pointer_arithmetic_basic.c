// Test basic pointer arithmetic
void putchar(int c);

int main() {
    int arr[10];
    
    // Initialize array
    arr[0] = 10;
    arr[1] = 20;
    arr[2] = 30;
    arr[3] = 40;
    arr[4] = 50;
    arr[5] = 60;
    arr[6] = 70;
    arr[7] = 80;
    arr[8] = 90;
    arr[9] = 100;
    
    // Test 1: Basic pointer arithmetic
    int *p = arr;
    int *q = p + 5;  // Should point to arr[5] = 60

    if (*q == 60) {
        putchar('1');  // Test 1 passed
    } else {
        putchar('N');  // Test 1 failed
    }

    // Test 2: Pointer increment
    p = arr;
    p = p + 3;  // Should point to arr[3] = 40

    if (*p == 40) {
        putchar('2');  // Test 2 passed
    } else {
        putchar('N');  // Test 2 failed
    }

    // Test 3: Pointer subtraction (decrement)
    p = &arr[7];  // Point to arr[7] = 80
    p = p - 2;    // Should point to arr[5] = 60

    if (*p == 60) {
        putchar('3');  // Test 3 passed
    } else {
        putchar('N');  // Test 3 failed
    }

    // Test 4: Array indexing (uses pointer arithmetic internally)
    if (arr[9] == 100) {
        putchar('4');  // Test 4 passed
    } else {
        putchar('N');  // Test 4 failed
    }

    // Test 5: Pointer through variable index
    int idx = 6;
    if (arr[idx] == 70) {
        putchar('5');  // Test 5 passed
    } else {
        putchar('N');  // Test 5 failed
    }
    
    putchar('\n');
    return 0;
}