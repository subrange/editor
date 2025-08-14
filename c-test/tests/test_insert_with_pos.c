void putchar(int c);

void insert_at_pos(char *arr, int pos, char ch) {
    int i = 3;  // Start from the end
    while (i > pos) {
        arr[i] = arr[i - 1];
        i = i - 1;
    }
    arr[pos] = ch;
}

int main() {
    char list[4];
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';
    list[3] = 'D';
    
    insert_at_pos(list, 1, 'X');
    
    putchar(list[0]);
    putchar(list[1]);
    putchar(list[2]);
    putchar(list[3]);
    putchar('\n');
    
    return 0;
}