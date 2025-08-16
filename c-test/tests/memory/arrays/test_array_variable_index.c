// Test array indexing with variable index
void putchar(int c);

int main() {
    int arr[10];
    
    // Initialize array manually
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
    
    // Test with variable index
    int idx = 6;
    int val = arr[idx];  // Should be 70
    
    if (val == 70) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}