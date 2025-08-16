void putchar(int c);

#include "once_test.h"
#include "once_test.h"  // Should be ignored due to #pragma once
#include "once_test.h"  // Should be ignored due to #pragma once

int main() {
    if (ONCE_VALUE == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    return 0;
}