void putchar(int c);

void minimal_insert(char *arr, int *dummy, int pos, char ch) {
    // Just store ch at arr[pos]
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
    putchar(list[2]);
    putchar(list[3]);
    putchar('\n');
    
    return 0;
}