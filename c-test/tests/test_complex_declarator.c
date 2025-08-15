void putchar(int c);

int main() {
    int arr[4] = {1, 2, 3, 4};

    // Test pointer to array
    int (*parr)[4] = &arr;
    
    // Access through pointer to array
    if ((*parr)[2] == 3) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    return 0;
}
