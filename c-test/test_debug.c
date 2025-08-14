void putchar(int c);

void minimal_insert(char *arr, int *dummy, int pos, char ch) {
    arr[pos] = ch;
}

int main() {
    char list[4];
    int dummy = 0;
    
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C'; 
    list[3] = 'D';

    minimal_insert(list, &dummy, 1, 'X');

    putchar(list[0]);
    putchar(list[1]);
    putchar('\n');
    
    return 0;
}
