// Compound literals in expressions
void putchar(int c);

typedef struct {
    int val;
} Box;

int sum_array(int *arr, int n) {
    int sum = 0;
    for (int i = 0; i < n; i++) {
        sum += arr[i];
    }
    return sum;
}

int get_box_value(Box *b) {
    return b->val;
}

int main() {
    // Test 1: Compound literal as function argument
    int result = sum_array((int[]){1, 2, 3, 4, 5}, 5);
    if (result == 15) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 2: Compound literal in conditional expression
    int x = 1;
    int *ptr = x ? (int[]){10} : (int[]){20};
    if (*ptr == 10) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 3: Compound literal with struct as function argument
    int val = get_box_value(&(Box){42});
    if (val == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 4: Compound literal in arithmetic expression
    int sum = ((int[]){5, 10})[0] + ((int[]){15, 20})[1];
    if (sum == 25) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}