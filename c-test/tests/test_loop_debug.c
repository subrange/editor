void putchar(int c);

void shift_right_debug(char *arr, int start, int end) {
    int i = end;
    putchar('S'); putchar('0' + i); putchar('\n');  // Start with i=3
    
    while (i > start) {
        putchar('B'); putchar('0' + i); putchar(':');
        putchar(arr[i - 1]); putchar('>'); putchar('0' + i); putchar('\n');
        
        arr[i] = arr[i - 1];
        
        // Show what's at position i now
        putchar('A'); putchar('0' + i); putchar('=');
        putchar(arr[i]); putchar('\n');
        
        i = i - 1;
    }
}

int main() {
    char list[10];
    
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';

    // Shift elements from position 1 to 3
    shift_right_debug(list, 1, 3);
    
    // Now insert X at position 1
    list[1] = 'X';

    // Print final result
    putchar('F'); putchar(':');
    putchar(list[0]);
    putchar(list[1]);
    putchar(list[2]);
    putchar(list[3]);
    putchar('\n');

    return 0;
}