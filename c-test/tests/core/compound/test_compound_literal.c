// Test compound literal support
#include <stdio.h>

typedef struct {
    int x;
    int y;
} point_t;

void putchar(int c);

int main() {
    // Test 1: Simple struct compound literal
    point_t p1 = (point_t){3, 4};
    if (p1.x == 3 && p1.y == 4) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 2: Compound literal in expression
    point_t *p2 = &(point_t){5, 6};
    if (p2->x == 5 && p2->y == 6) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 3: Array compound literal
    int *arr = (int[]){10, 20, 30};
    if (arr[0] == 10 && arr[1] == 20 && arr[2] == 30) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}