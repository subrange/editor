void putchar(int c);

void insert_char(char *arr, int *len, int pos, char ch) {
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

    // Print initial state
    for (int i = 0; i < len; i = i + 1) {
        putchar(list[i]);
    }
    putchar('\n');

    // Insert 'X' at position 1
    insert_char(list, &len, 1, 'X');

    // Print result - should be AXBC
    for (int i = 0; i < len; i = i + 1) {
        putchar(list[i]);
    }
    putchar('\n');

    return 0;
}