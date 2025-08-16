void putchar(int c);

void modify_via_pointer(char *arr, int pos, char val) {
    arr[pos] = val;
}

int main() {
    char list[10];
    
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';

    // Test 1: Direct modification
    list[3] = list[2];  // Copy C to position 3
    putchar(list[3]);  // Should print C
    putchar('\n');

    // Test 2: Modification via pointer parameter
    modify_via_pointer(list, 4, list[2]);  // Copy C to position 4 via function
    putchar(list[4]);  // Should print C
    putchar('\n');

    return 0;
}