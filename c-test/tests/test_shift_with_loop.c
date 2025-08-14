void putchar(int c);

void shift_with_loop(char *arr) {
    int i = 3;
    putchar('S'); putchar('\n');  // Start
    
    // First iteration: i=3, copy arr[2] to arr[3]
    if (i > 1) {
        putchar('1'); putchar(':');
        putchar('0' + i); putchar('\n');
        arr[i] = arr[i - 1];
        i = i - 1;
    }
    
    // Second iteration: i=2, copy arr[1] to arr[2]
    if (i > 1) {
        putchar('2'); putchar(':');
        putchar('0' + i); putchar('\n');
        arr[i] = arr[i - 1];
        i = i - 1;
    }
    
    putchar('E'); putchar('\n');  // End
}

int main() {
    char list[5];
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';
    
    shift_with_loop(list);
    list[1] = 'X';
    
    putchar(list[0]);
    putchar(list[1]);
    putchar(list[2]);
    putchar(list[3]);
    putchar('\n');
    
    return 0;
}