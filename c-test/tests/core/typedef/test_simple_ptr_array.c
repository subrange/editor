// Simplest test case for pointer to array
void putchar(int c);

int main() {
    int arr[3];
    int (*p)[3]; // Pointer to array of 3 ints
    p = &arr;    // This should work
    
    putchar('Y');
    return 0;
}