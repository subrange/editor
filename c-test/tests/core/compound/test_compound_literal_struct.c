// Compound literal tests with structs
void putchar(int c);

typedef struct {
    int a;
    int b;
} Pair;

typedef struct {
    char name[10];
    int age;
} Person;

typedef struct {
    int x;
    Pair p;
    int y;
} Nested;

int main() {
    // Test 1: Simple struct compound literal
    Pair *p1 = &(Pair){10, 20};
    if (p1->a == 10 && p1->b == 20) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 2: Direct struct assignment from compound literal
    Pair p2 = (Pair){30, 40};
    if (p2.a == 30 && p2.b == 40) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 3: Nested struct compound literal
    Nested n = (Nested){1, {2, 3}, 4};
    if (n.x == 1 && n.p.a == 2 && n.p.b == 3 && n.y == 4) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 4: Partial initialization (rest should be zero)
    Pair p3 = (Pair){.a = 50};
    if (p3.a == 50 && p3.b == 0) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}