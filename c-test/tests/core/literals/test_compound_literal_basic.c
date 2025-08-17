// Basic compound literal tests
void putchar(int c);

int main() {
    // Test 1: Simple int array compound literal
    int *arr1 = (int[]){1, 2, 3};
    if (arr1[0] == 1 && arr1[1] == 2 && arr1[2] == 3) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 2: Char array compound literal
    char *str = (char[]){'H', 'i', '\0'};
    if (str[0] == 'H' && str[1] == 'i' && str[2] == '\0') {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 3: Compound literal with single element
    int *single = (int[]){42};
    if (*single == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 4: Empty compound literal (zero-initialized)
    int *zeros = (int[3]){0};
    if (zeros[0] == 0 && zeros[1] == 0 && zeros[2] == 0) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}