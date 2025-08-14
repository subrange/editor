void putchar(int c);

void store_at_pos(char *arr, int pos, char ch) {
    arr[pos] = ch;
}

int main() {
    char list[3];
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';
    
    putchar(list[0]);
    putchar(list[1]);
    putchar(list[2]);
    putchar('\n');
    
    store_at_pos(list, 1, 'X');
    
    putchar(list[0]);
    putchar(list[1]);
    putchar(list[2]);
    putchar('\n');
    
    return 0;
}