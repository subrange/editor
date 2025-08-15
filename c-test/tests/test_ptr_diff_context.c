void putchar(int c);

int ptr_diff(int *a, int *b) {
    return (int)(a - b);
}

int main() {
    int arr[4];
    arr[0] = 1; arr[1] = 2; arr[2] = 3; arr[3] = 4;
    
    int diff = ptr_diff(&arr[3], &arr[0]);
    
    if (diff == 3) {
        putchar('Y');
    } else {
        putchar('0' + diff);
    }
    putchar('\n');
    
    return 0;
}
