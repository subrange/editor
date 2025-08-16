void putchar(int c);

void loop_func(char *arr) {
    int i = 3;
    
    while (i > 1) {
        putchar('0' + i);
        putchar('\n');
        i = i - 1;
    }
}

int main() {
    char list[5];
    list[0] = 'A';
    
    loop_func(list);
    
    putchar('D');
    putchar('\n');
    return 0;
}