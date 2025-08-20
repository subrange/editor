// Simple Forth-like stack calculator
// Demonstrates interactive input with TTY

#include <stdio.h>

int stack[100];
int sp = 0;

void push(int val) {
    if (sp < 100) {
        stack[sp++] = val;
    }
}

int pop() {
    if (sp > 0) {
        return stack[--sp];
    }
    return 0;
}

void print_stack() {
    puts("Stack:");
    if (sp == 0) {
        puts("  (empty)");
    } else {
        for (int i = 0; i < sp; i++) {
            putchar(' ');
            putchar(' ');
            // Print number (simplified - only handles small positive numbers)
            int n = stack[i];
            if (n == 0) {
                putchar('0');
            } else {
                char digits[10];
                int j = 0;
                while (n > 0) {
                    digits[j++] = '0' + (n % 10);
                    n = n / 10;
                }
                // Print in reverse
                while (j > 0) {
                    putchar(digits[--j]);
                }
            }
            putchar('\n');
        }
    }
}

int main() {
    puts("Simple Forth Calculator");
    puts("Commands: + - * / . (print top) s (show stack) q (quit)");
    puts("Enter numbers to push on stack");
    puts("================================");
    
    char buffer[20];
    int buf_pos = 0;
    int ch;
    
    while (1) {
        putchar('>');
        putchar(' ');
        
        buf_pos = 0;
        while (1) {
            ch = getchar();
            
            if (ch == '\n') {
                buffer[buf_pos] = 0;
                break;
            } else if (buf_pos < 19) {
                putchar(ch);  // Echo
                buffer[buf_pos++] = ch;
            }
        }
        
        // Process command
        if (buf_pos == 0) {
            continue;  // Empty line
        }
        
        if (buffer[0] == 'q') {
            puts("Goodbye!");
            break;
        } else if (buffer[0] == '+') {
            if (sp >= 2) {
                int b = pop();
                int a = pop();
                push(a + b);
                puts("ok");
            } else {
                puts("Stack underflow!");
            }
        } else if (buffer[0] == '-') {
            if (sp >= 2) {
                int b = pop();
                int a = pop();
                push(a - b);
                puts("ok");
            } else {
                puts("Stack underflow!");
            }
        } else if (buffer[0] == '*') {
            if (sp >= 2) {
                int b = pop();
                int a = pop();
                push(a * b);
                puts("ok");
            } else {
                puts("Stack underflow!");
            }
        } else if (buffer[0] == '/') {
            if (sp >= 2) {
                int b = pop();
                int a = pop();
                if (b != 0) {
                    push(a / b);
                    puts("ok");
                } else {
                    puts("Division by zero!");
                    push(a);  // Push back
                }
            } else {
                puts("Stack underflow!");
            }
        } else if (buffer[0] == '.') {
            if (sp > 0) {
                int val = stack[sp-1];
                // Print the number
                if (val == 0) {
                    putchar('0');
                } else {
                    char digits[10];
                    int j = 0;
                    int n = val;
                    if (n < 0) {
                        putchar('-');
                        n = -n;
                    }
                    while (n > 0) {
                        digits[j++] = '0' + (n % 10);
                        n = n / 10;
                    }
                    while (j > 0) {
                        putchar(digits[--j]);
                    }
                }
                putchar('\n');
            } else {
                puts("Stack empty!");
            }
        } else if (buffer[0] == 's') {
            print_stack();
        } else if (buffer[0] >= '0' && buffer[0] <= '9') {
            // Parse number
            int num = 0;
            for (int i = 0; i < buf_pos && buffer[i] >= '0' && buffer[i] <= '9'; i++) {
                num = num * 10 + (buffer[i] - '0');
            }
            push(num);
            puts("ok");
        } else {
            puts("Unknown command!");
        }
    }
    
    return 0;
}