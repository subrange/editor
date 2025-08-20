// Non-interactive Forth test
// Tests basic Forth operations without requiring user input

#include <stdio.h>

// Configuration
#define STACK_SIZE 100
#define MAX_WORDS 50
#define CODE_SIZE 500

// Data stack
int stack[STACK_SIZE];
int sp = 0;

// Dictionary - using parallel arrays instead of struct with array
char dict_names[1600];  // 50 * 32 = 1600 - Flattened array for names
int dict_is_prim[MAX_WORDS];
int dict_code_start[MAX_WORDS];
int dict_count = 0;

// Code storage
int code[CODE_SIZE];
int here = 0;

// State
int compile_mode = 0;

// Stack operations
void push(int val) {
    if (sp < STACK_SIZE) {
        stack[sp++] = val;
    }
}

int pop() {
    if (sp > 0) {
        return stack[--sp];
    }
    return 0;
}

// Print number
void print_num(int n) {
    if (n == 0) {
        putchar('0');
        return;
    }
    if (n < 0) {
        putchar('-');
        n = -n;
    }
    char buf[12];
    int i = 0;
    while (n > 0) {
        buf[i++] = '0' + (n % 10);
        n = n / 10;
    }
    while (i > 0) {
        putchar(buf[--i]);
    }
}

int main() {
    puts("Forth Test");
    
    // Test arithmetic
    puts("Testing 5 3 +:");
    push(5);
    push(3);
    int b = pop();
    int a = pop();
    push(a + b);
    print_num(pop());
    putchar('\n');
    
    puts("Testing 10 4 -:");
    push(10);
    push(4);
    b = pop();
    a = pop();
    push(a - b);
    print_num(pop());
    putchar('\n');
    
    puts("Testing 6 7 *:");
    push(6);
    push(7);
    b = pop();
    a = pop();
    push(a * b);
    print_num(pop());
    putchar('\n');
    
    puts("Testing 20 4 /:");
    push(20);
    push(4);
    b = pop();
    a = pop();
    if (b != 0) push(a / b);
    print_num(pop());
    putchar('\n');
    
    puts("Testing 17 5 MOD:");
    push(17);
    push(5);
    b = pop();
    a = pop();
    if (b != 0) push(a % b);
    print_num(pop());
    putchar('\n');
    
    // Test comparisons
    puts("Testing 5 5 =:");
    push(5);
    push(5);
    b = pop();
    a = pop();
    push(a == b ? -1 : 0);
    print_num(pop());
    putchar('\n');
    
    puts("Testing 3 7 <:");
    push(3);
    push(7);
    b = pop();
    a = pop();
    push(a < b ? -1 : 0);
    print_num(pop());
    putchar('\n');
    
    puts("Testing 10 5 >:");
    push(10);
    push(5);
    b = pop();
    a = pop();
    push(a > b ? -1 : 0);
    print_num(pop());
    putchar('\n');
    
    // Test stack operations
    puts("Testing DUP on 42:");
    push(42);
    push(stack[sp-1]); // DUP
    print_num(pop());
    putchar(' ');
    print_num(pop());
    putchar('\n');
    
    puts("Testing SWAP on 1 2:");
    push(1);
    push(2);
    if (sp >= 2) {
        int temp = stack[sp-1];
        stack[sp-1] = stack[sp-2];
        stack[sp-2] = temp;
    }
    print_num(pop());
    putchar(' ');
    print_num(pop());
    putchar('\n');
    
    puts("All tests complete!");
    
    return 0;
}