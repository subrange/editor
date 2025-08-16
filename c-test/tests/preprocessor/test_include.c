void putchar(int c);

#include "constants.h"

int main() {
    putchar(CONST_A);  // Should print 'A' (65)
    putchar(CONST_B);  // Should print 'B' (66)
    putchar(CONST_C);  // Should print 'C' (67)
    putchar('\n');
    return 0;
}