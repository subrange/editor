#include <stdio.h>

static int i = 0;

int main() {
    // Test static variable initialization
    if (i == 0) {
        putchar('Y');  // Should print 'Y' since i is initialized to 0
    } else {
        putchar('N');
    }

    // Increment static variable
    i++;

    // Test after increment
    if (i == 1) {
        putchar('Y');  // Should print 'Y' since i is now 1
    } else {
        putchar('N');
    }

    putchar('\n');

    return 0;
}