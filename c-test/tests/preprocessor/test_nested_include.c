void putchar(int c);

#include "nested1.h"  // This includes nested2.h

int main() {
    if (VALUE1 == 1) putchar('Y'); else putchar('N');
    if (VALUE2 == 2) putchar('Y'); else putchar('N');
    putchar('\n');
    return 0;
}