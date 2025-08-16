void putchar(int c);

int main() {
    int i = 3;
    
    while (i > 1) {
        putchar('0' + i);
        putchar('\n');
        i = i - 1;
    }
    
    putchar('D');
    putchar('\n');
    return 0;
}