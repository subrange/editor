#include <stdio.h>

typedef struct {
    short integer; 
    unsigned short frac; 
} q16_16_t; 

q16_16_t test;

int main() {
    test.integer = 1;
    test.frac = 0;

    if (test.integer == 1) putchar('Y');
    else putchar('N');

    if (test.frac == 0) putchar('Y');
    else putchar('N');

    return 0;
}