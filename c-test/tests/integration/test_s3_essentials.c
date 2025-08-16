// Stage 3 Essential Features Test
// Focuses on: structs, casts, pointer arithmetic, NULL

void putchar(int c);
void puts(char *s);

// Simple struct
struct Point {
    int x;
    int y;
};

// Nested struct
struct Rectangle {
    struct Point topLeft;
    struct Point bottomRight;
};

// Print helpers
void print_digit(int n) {
    if (n >= 0 && n <= 9) {
        putchar('0' + n);
    } else {
        putchar('?');
    }
}

int main() {
    puts("=== S3 ESSENTIALS ===");
    
    // Test 1: Basic struct
    puts("T1: Struct");
    struct Point p;
    p.x = 3;
    p.y = 4;
    print_digit(p.x);
    print_digit(p.y);
    putchar('\n');
    
    // Test 2: Struct pointer
    puts("T2: StructPtr");
    struct Point* pp = &p;
    pp->x = 5;
    pp->y = 6;
    print_digit(p.x);
    print_digit(p.y);
    putchar('\n');
    
    // Test 3: Nested struct
    puts("T3: Nested");
    struct Rectangle rect;
    rect.topLeft.x = 1;
    rect.topLeft.y = 2;
    rect.bottomRight.x = 7;
    rect.bottomRight.y = 8;
    print_digit(rect.topLeft.x);
    print_digit(rect.topLeft.y);
    print_digit(rect.bottomRight.x);
    print_digit(rect.bottomRight.y);
    putchar('\n');
    
    // Test 4: Type casting
    puts("T4: Cast");
    int num = 65;  // ASCII 'A'
    char ch = (char)num;
    putchar(ch);
    
    char c2 = 'B';
    int n2 = (int)c2;
    if (n2 == 66) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    // Test 5: Pointer casts
    puts("T5: PtrCast");
    int data = 42;
    void* vp = (void*)&data;
    int* ip = (int*)vp;
    if (*ip == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    // Test 6: NULL pointer
    puts("T6: NULL");
    int* null_ptr = (int*)0;
    if (null_ptr == (int*)0) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    int val = 99;
    int* valid = &val;
    if (valid != (int*)0) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    // Test 7: Pointer arithmetic
    puts("T7: PtrArith");
    int arr[5];
    int i;
    for (i = 0; i < 5; i = i + 1) {
        arr[i] = i * 10;
    }
    
    int* p1 = &arr[0];
    int* p2 = p1 + 3;  // Points to arr[3]
    print_digit(*p2 / 10);  // Should print 3
    
    int* p3 = p2 - 1;  // Points to arr[2]
    print_digit(*p3 / 10);  // Should print 2
    
    int diff = p2 - p1;  // Should be 3
    print_digit(diff);
    putchar('\n');
    
    // Test 8: Pointer comparisons
    puts("T8: PtrCmp");
    if (p2 > p1) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    if (p3 < p2) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    if (p1 <= p3 && p3 <= p2) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    // Test 9: Struct array
    puts("T9: StructArr");
    struct Point points[3];
    points[0].x = 1;
    points[0].y = 2;
    points[1].x = 3;
    points[1].y = 4;
    points[2].x = 5;
    points[2].y = 6;
    
    struct Point* sp = &points[1];
    print_digit(sp->x);
    print_digit(sp->y);
    
    sp = sp + 1;  // Move to points[2]
    print_digit(sp->x);
    print_digit(sp->y);
    putchar('\n');
    
    // Test 10: Address of struct member
    puts("T10: AddrOf");
    struct Rectangle r2;
    r2.topLeft.x = 7;
    int* xptr = &r2.topLeft.x;
    *xptr = 9;
    print_digit(r2.topLeft.x);
    
    struct Point* tlptr = &r2.topLeft;
    tlptr->y = 8;
    print_digit(r2.topLeft.y);
    putchar('\n');
    
    puts("=== END S3 ESSENTIALS ===");
    
    return 0;
}