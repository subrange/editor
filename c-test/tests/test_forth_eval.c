// Test Forth evaluation without interactive loop
#include <stdio.h>

// Minimal Forth for testing
int stack[10];
int sp = 0;

void push(int n) { 
    if (sp < 10) stack[sp++] = n; 
}

int pop() { 
    return sp > 0 ? stack[--sp] : 0; 
}

// Simple evaluator for testing
void eval_expr() {
    // Test: 5 3 + should give 8
    push(5);
    push(3);
    int b = pop();
    int a = pop();
    push(a + b);
    
    if (pop() == 8) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test: 10 4 - should give 6
    push(10);
    push(4);
    b = pop();
    a = pop();
    push(a - b);
    
    if (pop() == 6) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test: 6 7 * should give 42
    push(6);
    push(7);
    b = pop();
    a = pop();
    push(a * b);
    
    if (pop() == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test: DUP (duplicate top of stack)
    sp = 0;
    push(5);
    push(stack[sp-1]); // DUP
    
    if (sp == 2 && stack[0] == 5 && stack[1] == 5) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test: SWAP
    sp = 0;
    push(3);
    push(7);
    // SWAP
    int temp = stack[sp-1];
    stack[sp-1] = stack[sp-2];
    stack[sp-2] = temp;
    
    if (stack[0] == 7 && stack[1] == 3) {
        putchar('Y');
    } else {
        putchar('N');
    }
}

int main() {
    eval_expr();
    putchar('\n');
    return 0;
}