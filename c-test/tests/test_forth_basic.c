// Basic Forth test - verifies core functionality
#include <stdio.h>

// Simplified Forth for testing
int stack[10];
int sp = 0;

void push(int n) { 
    if (sp < 10) stack[sp++] = n; 
}

int pop() { 
    return sp > 0 ? stack[--sp] : 0; 
}

void test_arithmetic() {
    // Test 5 + 3 = 8
    push(5);
    push(3);
    int b = pop();
    int a = pop();
    push(a + b);
    
    if (pop() == 8) {
        putchar('Y'); // Yes - addition works
    } else {
        putchar('N');
    }
    
    // Test 10 - 4 = 6
    push(10);
    push(4);
    b = pop();
    a = pop();
    push(a - b);
    
    if (pop() == 6) {
        putchar('Y'); // Yes - subtraction works
    } else {
        putchar('N');
    }
    
    // Test 6 * 7 = 42
    push(6);
    push(7);
    b = pop();
    a = pop();
    push(a * b);
    
    if (pop() == 42) {
        putchar('Y'); // Yes - multiplication works
    } else {
        putchar('N');
    }
}

void test_stack() {
    sp = 0; // Reset stack
    
    // Test DUP
    push(5);
    int val = stack[sp-1];
    push(val); // DUP
    
    if (sp == 2 && stack[0] == 5 && stack[1] == 5) {
        putchar('Y'); // DUP works
    } else {
        putchar('N');
    }
    
    // Test SWAP
    sp = 0;
    push(3);
    push(7);
    int temp = stack[sp-1];
    stack[sp-1] = stack[sp-2];
    stack[sp-2] = temp; // SWAP
    
    if (stack[0] == 7 && stack[1] == 3) {
        putchar('Y'); // SWAP works
    } else {
        putchar('N');
    }
}

int main() {
    test_arithmetic();
    test_stack();
    putchar('\n');
    return 0;
}