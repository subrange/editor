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

void delete_char(char *arr, int *len, int pos) {
    int i = pos;
    while (i < *len - 1) {
        arr[i] = arr[i + 1];
        i = i + 1;
    }
    *len = *len - 1;
}

void print_chars(char *arr, int len) {
    int i = 0;
    while (i < len) {
        putchar(arr[i]);
        i = i + 1;
    }
    putchar('\n');
}

int main() {
    char list[10];
    int len = 3;

    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';

    print_chars(list, len);

    insert_char(list, &len, 1, 'X');
    print_chars(list, len);

    delete_char(list, &len, 1);
    print_chars(list, len);

    return 0;
}