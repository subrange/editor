void putchar(int c);

void test_func(char *arr) {
    // Just try to read arr[1]
    putchar(arr[1]);
    putchar('\n');
}

int main() {
    char list[3];
    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';
    
    test_func(list);
    
    return 0;
}