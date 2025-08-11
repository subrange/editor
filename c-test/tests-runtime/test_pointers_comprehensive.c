// Comprehensive test for pointer operations
void putchar(int c);

// Global for testing global pointers
int global_val = 42;

int main() {
    // Test 1: Basic address-of and dereference
    int x = 10;
    int *p = &x;
    int val = *p;  // Should be 10
    
    // Test 2: Modify through pointer
    *p = 20;
    if (x == 20) {
        putchar('1');  // Test 1 passed
    } else {
        putchar('N');  // Test 1 failed
    }
    
    // Test 3: Pointer to global
    int *gp = &global_val;
    if (*gp == 42) {
        putchar('2');  // Test 2 passed
    } else {
        putchar('N');  // Test 1 failed
    }
    
    // Test 4: Modify global through pointer
    *gp = 100;
    if (global_val == 100) {
        putchar('3');  // Test 3 passed
    } else {
         putchar('N');  // Test 1 failed
     }
    
    // Test 5: Multiple levels of indirection
    int y = 5;
    int *py = &y;
    int **ppy = &py;
    **ppy = 15;
    if (y == 15) {
        putchar('4');  // Test 4 passed
    } else {
             putchar('N');  // Test 1 failed
         }
    
    // Test 6: Pointer arithmetic (basic)
    int a = 1;
    int b = 2;
    int *pa = &a;
    int *pb = &b;
    
    // Swap using pointers
    int temp = *pa;
    *pa = *pb;
    *pb = temp;
    
    if (a == 2) {
        if (b == 1) {
            putchar('5');  // Test 5 passed
        } else {
            putchar('N');
            putchar('1');  // Test 5 failed
        }
    } else {
        putchar('N');
        putchar('2');  // Test 5 failed
     }
    
    putchar('\n');
    return 0;
}