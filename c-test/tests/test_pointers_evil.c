void putchar(int c);

int global_x = 123;
char global_str[] = "Hello";

int* return_ptr(int *p) { return p; }

void increment_all(int *arr, int n) {
    int i = 0;
    while (i < n) {
        arr[i] = arr[i] + 1;
        i = i + 1;
    }
}

void swap_ints(int *a, int *b) {
    int t = *a;
    *a = *b;
    *b = t;
}

void touch_void(void *ptr) {
    *((int*)ptr) = 999;
}

int ptr_diff(int *a, int *b) {
    return (int)(a - b);
}

int main() {
    int a = 10;
    int *pa = &a;

    // 1. Basic dereference/local
    if (*pa == 10) putchar('Y'); else putchar('N');

    // 2. Modify local via pointer
    *pa = 20;
    if (a == 20) putchar('Y'); else putchar('N');

    // 3. Global var via pointer
    int *pg = &global_x;
    if (*pg == 123) putchar('Y'); else putchar('N');

    // 4. Modify global var via pointer
    *pg = *pg + 1;
    if (global_x == 124) putchar('Y'); else putchar('N');

    // 5. Pointer to pointer
    int *pb = &a;
    int **ppb = &pb;
    **ppb = **ppb + 3;
    if (a == 23) putchar('Y'); else putchar('N');

    // 6. Array + pointer arithmetic
    int arr[4];
    arr[0] = 1; arr[1] = 2; arr[2] = 3; arr[3] = 4;
    int *p = arr;
    if (*(p + 2) == 3) putchar('Y'); else putchar('N');
    *(p + 1) = 42;
    if (arr[1] == 42) putchar('Y'); else putchar('N');

    // 7. Array of pointers
    int x = 5; int y = 6; int z = 7;
    int *ptrs[3];
    ptrs[0] = &x; ptrs[1] = &y; ptrs[2] = &z;
    if (*ptrs[1] == 6) putchar('Y'); else putchar('N');
    *ptrs[0] = 99;
    if (x == 99) putchar('Y'); else putchar('N');

    // 8. Pointer to array
    int (*parr)[4] = &arr;
    if ((*parr)[3] == 4) putchar('Y'); else putchar('N');

    // 9. Void pointer casting
    void *vp = &y;
    touch_void(vp);
    if (y == 999) putchar('Y'); else putchar('N');

    // 10. Struct-like navigation (manual)
    int n1_val = 11;
    int *n1_next_val;
    int n2_val = 22;
    int *n2_next_val = 0;
    n1_next_val = &n2_val;
    if (*n1_next_val == 22) putchar('Y'); else putchar('N');

    // 11. Return pointer from function
    int tval = 77;
    int *tp = return_ptr(&tval);
    if (*tp == 77) putchar('Y'); else putchar('N');

    // 12. Pass pointer to function
    int nums[3];
    nums[0] = 5; nums[1] = 6; nums[2] = 7;
    increment_all(nums, 3);
    if (nums[0] == 6 && nums[1] == 7 && nums[2] == 8)
        putchar('Y');
    else
        putchar('N');

    // 13. Pointer comparison & diff
    int diff = ptr_diff(&arr[3], &arr[0]);
    if (diff == 3) putchar('Y'); else putchar('N');

    // 14. Swap via pointer
    int m = 1; int n = 2;
    swap_ints(&m, &n);
    if (m == 2 && n == 1) putchar('Y'); else putchar('N');

    putchar('\n');
    return 0;
}