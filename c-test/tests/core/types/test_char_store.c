void putchar(int c);

void store_char(char *arr, char ch) {
    arr[1] = ch;
}

int main() {
    char list[3];
    list[0] = 'A';
    list[1] = 'B';  
    list[2] = 'C';
    
    store_char(list, 'X');
    
    putchar(list[0]);
    putchar(list[1]);
    putchar(list[2]);
    putchar('\n');
    
    return 0;
}