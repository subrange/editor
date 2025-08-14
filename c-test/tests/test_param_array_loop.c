void putchar(int c);

void test_loop(char *arr) {
    // Try to copy arr[2] to arr[3] and arr[1] to arr[2]
    int i = 3;
    while (i > 1) {
        putchar('C'); putchar('0' + i); putchar(':');
        putchar(arr[i - 1]); putchar('\n');
        
        arr[i] = arr[i - 1];
        
        putchar('W'); putchar('0' + i); putchar(':');
        putchar(arr[i]); putchar('\n');
        
        i = i - 1;
    }
}

int main() {
    char list[5];
    
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';
    
    putchar('B'); putchar(':');
    putchar(list[0]);
    putchar(list[1]);
    putchar(list[2]);
    putchar('\n');
    
    test_loop(list);
    
    putchar('A'); putchar(':');
    putchar(list[0]);
    putchar(list[1]);
    putchar(list[2]);
    putchar(list[3]);
    putchar('\n');
    
    return 0;
}