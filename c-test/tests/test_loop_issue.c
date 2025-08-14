void putchar(int c);

void shift_right(char *arr, int start, int end) {
    int i = end;
    while (i > start) {
        arr[i] = arr[i - 1];
        i = i - 1;
    }
}

int main() {
    char list[10];
    
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';

    // Shift elements from position 1 to 3 (move B and C one position right)
    shift_right(list, 1, 3);
    
    // Now insert X at position 1
    list[1] = 'X';

    // Print result - should be AXBC
    putchar(list[0]);
    putchar(list[1]);
    putchar(list[2]);
    putchar(list[3]);
    putchar('\n');

    return 0;
}