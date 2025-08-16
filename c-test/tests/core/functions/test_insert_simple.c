void putchar(int c);

void insert_at_one(char *arr, char ch) {
    // Simplified version - just store at position 1
    int i = 3;  // Start from the end
    while (i > 1) {
        arr[i] = arr[i - 1];
        i = i - 1;
    }
    arr[1] = ch;
}

int main() {
    char list[4];
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';
    list[3] = 'D';
    
    insert_at_one(list, 'X');
    
    putchar(list[0]);
    putchar(list[1]);
    putchar(list[2]);
    putchar(list[3]);
    putchar('\n');
    
    return 0;
}