void putchar(int c);

void insert_with_len(char *arr, int *len, int pos, char ch) {
    int i = *len;
    while (i > pos) {
        arr[i] = arr[i - 1];
        i = i - 1;
    }
    arr[pos] = ch;
    *len = *len + 1;
}

int main() {
    char list[10];
    int len = 3;
    
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';
    
    insert_with_len(list, &len, 1, 'X');
    
    int i = 0;
    while (i < len) {
        putchar(list[i]);
        i = i + 1;
    }
    putchar('\n');
    
    return 0;
}