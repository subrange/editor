// Test basic pointer arithmetic
void putchar(int c);

int main() {
    int arr[10];

    // Initialize array
//    arr[0] = 10;
//    arr[1] = 20;
//    arr[2] = 30;
//    arr[3] = 40;
//    arr[4] = 50;
//    arr[5] = 60;
//    arr[6] = 70;
//    arr[7] = 80;
//    arr[8] = 90;
//    arr[9] = 100;
//
//    // Test 1: Basic pointer arithmetic
    int *p = arr;
    int *q = p + 5;  // Should point to arr[5] = 60


    putchar('\n');
    return 0;
}