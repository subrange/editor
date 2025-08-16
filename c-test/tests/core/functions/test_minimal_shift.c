void putchar(int c);

void shift_test(char *arr) {
    // Just copy arr[2] to arr[3] and arr[1] to arr[2]
    arr[3] = arr[2];
    arr[2] = arr[1];
}

int main() {
    char list[5];
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';
    
    shift_test(list);
    list[1] = 'X';
    
    putchar(list[0]);
    putchar(list[1]);
    putchar(list[2]);
    putchar(list[3]);
    putchar('\n');
    
    return 0;
}