void putchar(int c);

void shift_func(char *arr) {
    int i = 3;
    
    while (i > 1) {
        putchar('I'); putchar('0' + i); putchar('\n');
        arr[i] = arr[i - 1];  // Same as test_real_while_loop
        i = i - 1;
    }
}

int main() {
    char list[5];
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';
    
    shift_func(list);
    
    putchar(list[0]);
    putchar(list[1]);
    putchar(list[2]);
    putchar(list[3]);
    putchar('\n');
    return 0;
}