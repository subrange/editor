void putchar(int c);

int main() {
    char arr[5];
    
    // Initialize array
    arr[0] = 'A';
    arr[1] = 'B';
    arr[2] = 'C';
    
    // Test 1: Write arr[2] to arr[3] directly
    arr[3] = arr[2];
    putchar(arr[3]); // Should print C
    putchar('\n');
    
    // Test 2: Write arr[1] to arr[2] directly
    arr[2] = arr[1];
    putchar(arr[2]); // Should print B
    putchar('\n');
    
    // Test 3: Now check what's at arr[3]
    putchar(arr[3]); // Should still be C
    putchar('\n');
    
    // Test 4: Do both operations in a loop
    int i = 3;
    while (i > 1) {
        putchar('L'); putchar('0' + i); putchar(':');
        putchar(arr[i - 1]); putchar('>'); putchar('0' + i); putchar('\n');
        arr[i] = arr[i - 1];
        i = i - 1;
    }
    
    // Check final state - should be ABBB
    putchar('F'); putchar(':');
    putchar(arr[0]);
    putchar(arr[1]);
    putchar(arr[2]);
    putchar(arr[3]);
    putchar('\n');
    
    return 0;
}