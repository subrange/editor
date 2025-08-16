void putchar(int c);

void loop_func(char *arr) {
    int i = 3;
    
    while (i > 1) {
        putchar('I'); putchar('='); putchar('0' + i); putchar('\n');
        arr[i] = 'X';
        i = i - 1;
        putchar('A'); putchar('='); putchar('0' + i); putchar('\n');
    }
}

int main() {
    char list[5];
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';
    list[3] = 'D';
    list[4] = 'E';
    
    loop_func(list);
    
    // Check what got written
    putchar('R'); putchar(':');
    putchar(list[0]);
    putchar(list[1]);
    putchar(list[2]);
    putchar(list[3]);
    putchar(list[4]);
    putchar('\n');
    return 0;
}