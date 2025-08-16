void putchar(int c);

int ptr_diff(int *a, int *b) {
    return (int)(a - b);
}

int main() {
    int arr[4];
    int diff = ptr_diff(&arr[3], &arr[0]);
    
    if (diff == 3) {
        putchar('Y');
    } else {
        putchar('0' + diff);  // Output the actual difference
    }
    putchar('\n');
    
    return 0;
}
