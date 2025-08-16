// Stage 2 Kitchen Sink Test - Tests EVERYTHING the compiler can handle
// Including nasty edge cases and complex pointer operations

void putchar(int c);
void puts(char *s);

// Global variables of various types
int global_int = 42;
char global_char = 'G';
char global_string[] = "GLOBAL";  // THIS SHOULD WORK - if not, it's a BUG!
int global_array[3] = {100, 200, 300};

// Helper function to print a number (0-9)
void print_digit(int n) {
    if (n >= 0 && n <= 9) {
        putchar('0' + n);
    } else {
        putchar('?');
    }
}

// Test function with multiple parameters and pointer arithmetic
int process_array(char *arr, int len, int offset) {
    int sum = 0;
    int i = 0;
    
    // Test while loop with complex condition
    while (i < len && arr[i] != 0) {
        // Test pointer arithmetic and dereferencing
        char val = *(arr + i + offset);
        sum = sum + val;
        i = i + 1;
    }
    
    return sum;
}

// Test recursion (simple factorial)
int factorial(int n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

// Test complex pointer operations including the nasty ***&& case
void test_evil_pointers(int *p) {
    // Test: *p
    *p = 123;
    
    // Test: **&p (should be same as *p)
    int **pp = &p;
    **pp = 456;
    
    // Test: ***&&p (should still be same as *p)
    int ***ppp = &pp;
    ***ppp = 789;
    
    // Even nastier: ****&&&p
    int ****pppp = &ppp;
    ****pppp = 999;
    
    putchar('P');
    print_digit((*p) / 100);
    print_digit(((*p) / 10) % 10);
    print_digit((*p) % 10);
    putchar('\n');
}

// Test function that modifies array through pointer
void shift_array(char *arr, int len) {
    int i = len - 1;
    while (i > 0) {
        arr[i] = arr[i - 1];
        i = i - 1;
    }
    arr[0] = 'X';
}

// Test nested function calls and complex expressions
int complex_calc(int a, int b, int c) {
    // Test operator precedence and associativity
    int result = a + b * c - (a * b + c) / 2;
    
    // Test bitwise operations
    result = result ^ 0x0F;
    result = result & 0xFF;
    result = result | 0x40;
    
    // Test shifts
    result = result << 1;
    result = result >> 2;
    
    return result;
}

// Main test driver
int main() {
    puts("=== S2 KITCHEN SINK TEST ===");
    
    // Test 1: Local variables and arrays
    puts("T1: Locals");
    char local_array[10];
    local_array[0] = 'L';
    local_array[1] = 'O';
    local_array[2] = 'C';
    local_array[3] = 'A';
    local_array[4] = 'L';
    local_array[5] = 0;
    puts(local_array);
    
    // Test 2: Global access
    puts("T2: Globals");
    putchar(global_char);
    putchar('\n');
    puts(global_string);  // BUG: This prints nothing!
    print_digit(global_int / 10);
    print_digit(global_int % 10);
    putchar('\n');
    
    // Test 3: Complex pointer operations
    puts("T3: Evil ptrs");
    int x = 0;
    test_evil_pointers(&x);
    
    // Test 4: Array operations and loops
    puts("T4: Arrays");
    char test_arr[5];
    test_arr[0] = 'A';
    test_arr[1] = 'B';
    test_arr[2] = 'C';
    test_arr[3] = 'D';
    test_arr[4] = 0;
    
    // Test array indexing with complex expressions
    int idx = 2;
    char c = test_arr[idx + 1];
    putchar(c);  // Should print 'D'
    putchar('\n');
    
    // Test 5: Function calls with arrays
    puts("T5: Shift");
    shift_array(test_arr, 4);
    puts(test_arr);
    
    // Test 6: Nested loops
    puts("T6: Nested");
    int i = 0;
    while (i < 2) {
        int j = 0;
        while (j < 3) {
            putchar('0' + i);
            putchar('0' + j);
            putchar(' ');
            j = j + 1;
        }
        putchar('\n');
        i = i + 1;
    }
    
    // Test 7: Conditionals and ternary
    puts("T7: Conds");
    int a = 5;
    int b = 3;
    
    if (a > b) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Multiple conditions
    if (a > 0 && b > 0 && a > b) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Complex condition with ||
    if ((a < 0) || (b < 0) || (a == 5)) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    // Test 8: Arithmetic and bitwise operations
    puts("T8: Math");
    int result = complex_calc(10, 20, 3);
    print_digit((result / 10) % 10);
    print_digit(result % 10);
    putchar('\n');
    
    // Test 9: Recursion (factorial of 5 = 120)
    puts("T9: Recur");
    int fact = factorial(5);
    print_digit(fact / 100);
    print_digit((fact / 10) % 10);
    print_digit(fact % 10);
    putchar('\n');
    
    // Test 10: String operations and pointer arithmetic
    puts("T10: Strings");
    char *str = "HELLO";
    char *p = str;
    while (*p) {
        putchar(*p);
        p = p + 1;
    }
    putchar('\n');
    
    // Test with string literal in array initialization
    char str_arr[] = "INIT";
    puts(str_arr);
    
    // Test 11: Complex expressions (no post-increment)
    puts("T11: Complex");
    int counter = 0;
    int vals[3];
    vals[counter] = 10;
    counter = counter + 1;
    vals[counter] = 20;
    counter = counter + 1;
    vals[counter] = 30;
    counter = counter + 1;
    print_digit(counter);
    putchar(' ');
    print_digit(vals[0] / 10);
    print_digit(vals[1] / 10);
    print_digit(vals[2] / 10);
    putchar('\n');
    
    // Test 12: For loops (desugared to while)
    puts("T12: For");
    for (int k = 0; k < 5; k = k + 1) {
        putchar('0' + k);
    }
    putchar('\n');
    
    // Test 13: Break and continue in loops
    puts("T13: Break");
    i = 0;
    while (1) {
        if (i >= 3) {
            break;
        }
        putchar('B');
        putchar('0' + i);
        i = i + 1;
    }
    putchar('\n');
    
    // Test 14: Pointer to pointer operations
    puts("T14: PtrPtr");
    int value = 42;
    int *ptr1 = &value;
    int **ptr2 = &ptr1;
    **ptr2 = 84;
    print_digit(value / 10);
    print_digit(value % 10);
    putchar('\n');
    
    // Test 15: Mixed pointer and array operations
    puts("T15: Mixed");
    char mix[4] = {'M', 'I', 'X', 0};
    char *mp = mix;
    putchar(mp[0]);
    putchar(*(mp + 1));
    putchar(mix[2]);
    putchar('\n');
    
    // Test 16: Function pointer (if supported - might fail)
    // Commenting out as function pointers aren't supported yet
    // void (*fptr)(int) = print_digit;
    // fptr(7);
    
    // Test 17: Unary operations
    puts("T17: Unary");
    int un = 5;
    un = -un;  // Should be -5
    un = ~un;  // Should be 4 (bitwise NOT of -5)
    un = !un;  // Should be 0 (logical NOT of non-zero)
    un = !un;  // Should be 1 (logical NOT of 0)
    print_digit(un);
    putchar('\n');
    
    // Test 18: Character literals and escape sequences
    puts("T18: Chars");
    putchar('A');
    putchar('\n');
    putchar('\t');  // Tab
    putchar('T');
    putchar('A');
    putchar('B');
    putchar('\n');
    
    // Test 19: Do-while loop
    puts("T19: DoWhile");
    int dw = 0;
    do {
        putchar('D');
        putchar('0' + dw);
        dw = dw + 1;
    } while (dw < 3);
    putchar('\n');
    
    // Test 20: Multiple if-else (switch not supported yet)
    puts("T20: IfElse");
    int sw = 2;
    if (sw == 1) {
        putchar('O');
        putchar('N');
        putchar('E');
    } else if (sw == 2) {
        putchar('T');
        putchar('W');
        putchar('O');
    } else if (sw == 3) {
        putchar('T');
        putchar('H');
        putchar('R');
    } else {
        putchar('D');
        putchar('E');
        putchar('F');
    }
    putchar('\n');
    
    // Final message
    puts("=== END KITCHEN SINK ===");
    
    return 0;
}