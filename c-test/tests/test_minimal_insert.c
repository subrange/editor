void putchar(int c);

void minimal_insert(char *arr, int *dummy, int pos, char ch) {
    arr[pos] = ch;
}

void minimal_insert2(char *arr, int pos, char ch) {
    arr[pos] = ch;
}

void minimal_insert3(char *arr, char ch) {
    arr[1] = ch;
}

void minimal_insert4(char *arr) {
    arr[1] = 'X';
}

int main() {
    char list[4];
    int dummy = 0;
    
    list[0] = 'A';
    list[1] = 'B';

//    minimal_insert(list, &dummy, 1, 'X'); // Outputs "\x01X"
//    minimal_insert2(list, 1, 'X'); // Outputs '\x00X'
//    minimal_insert3(list, 'X'); // Outputs "XX"
    minimal_insert4(list); // Works

    putchar(list[0]);
    putchar(list[1]);
    putchar('\n');
    
    return 0;
}