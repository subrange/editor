void putchar(int c);
int puts(char* c);

void minimal_insert4(char *arr, int *dummy, int pos, char ch) {
    arr[pos] = ch;
}

void minimal_insert3(char *arr, int pos, char ch) {
    arr[pos] = ch;
}

void minimal_insert2(char *arr, char ch) {
    arr[1] = ch;
}

void minimal_insert1(char *arr) {
    arr[1] = 'X';
}

int main() {
    char list[4];
    int dummy = 0;
    
    list[0] = '0';
    list[1] = '0';

    minimal_insert1(list);
    if (list[1] == 'X') {
        puts("Pass: 1");
    } else {
        puts("Fail: 1");
    }

    minimal_insert2(list, 'Y');
    if (list[1] == 'Y') {
        puts("Pass: 2");
    } else {
        puts("Fail: 2");
    }

    minimal_insert3(list, 1, 'Z');
    if (list[1] == 'Z') {
        puts("Pass: 3");
    } else {
        puts("Fail: 3");
    }

    minimal_insert4(list, &dummy, 1, 'W');
    if (list[1] == 'W') {
        puts("Pass: 4");
    } else {
        puts("Fail: 4");
    }

    putchar(list[0]);
    putchar(list[1]);
    putchar('\n');
    
    return 0;
}