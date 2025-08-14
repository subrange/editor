void putchar(int c);

void test_func(char *arr) {
    int i = 3;
    
    while (i > 1) {
        putchar('I'); putchar('0' + i); putchar('\n');
        char c = arr[i - 1];  // Read from array
        putchar('C'); putchar('='); putchar(c); putchar('\n');
        i = i - 1;
    }
}

int main() {
    char list[5];
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';
    list[3] = 'D';
    
    test_func(list);
    
    putchar('D'); putchar('O'); putchar('N'); putchar('E'); putchar('\n');
    return 0;
}