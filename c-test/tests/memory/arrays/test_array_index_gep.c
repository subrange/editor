// Test array indexing via GEP
void putchar(int c);

int main() {
    int arr[10];
    
    // Initialize array
    arr[0] = 0;
    arr[1] = 1;
    arr[2] = 2;
    arr[3] = 3;
    arr[4] = 4;
    arr[5] = 5;
    arr[6] = 6;
    arr[7] = 7;
    arr[8] = 8;
    arr[9] = 9;
    
    // Test simple indexing
    if (arr[5] == 5) {
        putchar('Y');  // Yes
    } else {
        putchar('N');  // No
    }
    
    // Test pointer arithmetic equivalence (arr[i] == *(arr + i))
    int *p = arr;
    if (*(p + 7) == 7) {
        putchar('Y');  // Yes
    } else {
        putchar('N');  // No
    }
    
    // Test that array indexing properly scales by element size
    int *q = &arr[3];
    if (*q == 3) {
        putchar('Y');  // Yes
    } else {
        putchar('N');  // No
    }
    
    putchar('\n');
    return 0;
}