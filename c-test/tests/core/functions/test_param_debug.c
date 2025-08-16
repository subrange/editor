void putchar(int c);

void debug_params(char *arr, int *len, int pos, char ch) {
    putchar('p');
    putchar('o');
    putchar('s');
    putchar('=');
    putchar('0' + pos);
    putchar(' ');
    putchar('c');
    putchar('h');
    putchar('=');
    putchar(ch);
    putchar('\n');
}

int main() {
    char list[10];
    int len = 3;
    
    debug_params(list, &len, 1, 'X');
    
    return 0;
}