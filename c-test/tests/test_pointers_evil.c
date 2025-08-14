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
    int checksum = 0;

    /* 1. Basic dereference/local */
    int a = 10;
    int *pa = &a;
    checksum = checksum ^ *pa;

    /* 2. Modify local via pointer */
    *pa = 20;
    checksum = checksum ^ a;

    /* 3. Global var via pointer */
    int *pg = &global_x;
    checksum = checksum ^ *pg;

    /* 4. Modify global var via pointer */
    *pg = *pg + 1;
    checksum = checksum ^ global_x;

    /* 5. Pointer to pointer */
    int *pb = &a;
    int **ppb = &pb;
    **ppb = **ppb + 3;
    checksum = checksum ^ a;

    /* 6. Array + pointer arithmetic */
    int arr[4];
    arr[0] = 1; arr[1] = 2; arr[2] = 3; arr[3] = 4;
    int *p = arr;
    checksum = checksum ^ *(p+2);
    *(p+1) = 42;
    checksum = checksum ^ arr[1];

    /* 7. Array of pointers */
    int x = 5; int y = 6; int z = 7;
    int *ptrs[3];
    ptrs[0] = &x; ptrs[1] = &y; ptrs[2] = &z;
    checksum = checksum ^ *ptrs[1];
    *ptrs[0] = 99;
    checksum = checksum ^ x;

    /* 8. Pointer to array */
    int (*parr)[4] = &arr;
    checksum = checksum ^ (*parr)[3];

    /* 9. Void pointer casting */
    void *vp = &y;
    touch_void(vp);
    checksum = checksum ^ y;

    /* 10. Struct-like navigation (manual) */
    /* Simulate struct Node { int val; struct Node *next; }; */
    int n1_val = 11;
    int *n1_next_val;
    int n2_val = 22;
    int *n2_next_val = 0;
    n1_next_val = &n2_val;
    checksum = checksum ^ *n1_next_val;

    /* 11. Return pointer from function */
    int tval = 77;
    checksum = checksum ^ *return_ptr(&tval);

    /* 12. Pass pointer to function */
    int nums[3];
    nums[0] = 5; nums[1] = 6; nums[2] = 7;
    increment_all(nums, 3);
    checksum = checksum ^ (nums[0] + nums[1] + nums[2]);

    /* 13. Pointer comparison & diff */
    checksum = checksum ^ ptr_diff(&arr[3], &arr[0]);

    /* 14. Swap via pointer */
    int m = 1; int n = 2;
    swap_ints(&m, &n);
    checksum = checksum ^ (m*10 + n);

    /* Output checksum as decimal */
    int out = checksum;
    char buf[12];
    int idx = 0;
    if (out == 0) { putchar('0'); putchar('\n'); return 0; }
    if (out < 0) { putchar('-'); out = -out; }
    while (out > 0) {
        buf[idx] = '0' + (out % 10);
        idx = idx + 1;
        out = out / 10;
    }
    while (idx > 0) {
        idx = idx - 1;
        putchar(buf[idx]);
    }
    putchar('\n');

    return 0;
}