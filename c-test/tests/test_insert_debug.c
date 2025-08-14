void putchar(int c);

void insert_char(char *arr, int *len, int pos, char ch) {
    // Debug: print inputs
    putchar('L'); putchar(':'); putchar('0' + *len); putchar(' ');
    putchar('P'); putchar(':'); putchar('0' + pos); putchar(' ');
    putchar('C'); putchar(':'); putchar(ch); putchar('\n');
    
    int i = *len;
    while (i > pos) {
        putchar('M'); putchar(':'); putchar('0' + i); putchar('\n'); // Debug move
        arr[i] = arr[i - 1];
        i = i - 1;
    }
    arr[pos] = ch;
    *len = *len + 1;
    
    // Debug: print final length
    putchar('F'); putchar(':'); putchar('0' + *len); putchar('\n');
}

int main() {
    char list[10];
    int len = 3;
    
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';
    
    // Print initial
    for (int i = 0; i < len; i = i + 1) {
        putchar(list[i]);
    }
    putchar('\n');
    
    // Insert 'X' at position 1
    insert_char(list, &len, 1, 'X');
    
    // Print result
    for (int i = 0; i < len; i = i + 1) {
        putchar(list[i]);
    }
    putchar('\n');
    
    return 0;
}