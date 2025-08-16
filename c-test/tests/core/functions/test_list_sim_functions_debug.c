void putchar(int c);
int puts(char* c);

void insert_char(char *arr, int *len, int pos, char ch) {
    int i = *len;
    putchar('S'); putchar('0' + i); putchar('\n'); // Debug: start value of i
    while (i > pos) {
        putchar('B'); putchar('0' + i); putchar(':');
        putchar(arr[i - 1]); putchar('\n'); // Debug: what we're copying
        arr[i] = arr[i - 1];
        i = i - 1;
    }
    putchar('W'); putchar(':'); putchar(ch); putchar('@'); putchar('0' + pos); putchar('\n'); // Debug: writing ch at pos
    arr[pos] = ch;
    *len = *len + 1;
    
    // Debug: verify what's actually in the array now
    putchar('V'); putchar(':');
    for (int j = 0; j < *len; j = j + 1) {
        putchar(arr[j]);
    }
    putchar('\n');
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
    putchar('D'); putchar('1'); putchar('\n'); // Debug marker

    insert_char(list, &len, 1, 'X');
    putchar('D'); putchar('2'); putchar('\n'); // Debug marker
    
    // Manual print to see what's actually in the array
    putchar('[');
    for (int i = 0; i < len; i = i + 1) {
        putchar(list[i]);
    }
    putchar(']');
    putchar('\n');
    
    // Add null terminator before calling puts!
    list[len] = '\0';

    puts(list);  // This is line 44 in original
    putchar('D'); putchar('3'); putchar('\n'); // Debug marker

    delete_char(list, &len, 1);
    print_chars(list, len);

    return 0;
}