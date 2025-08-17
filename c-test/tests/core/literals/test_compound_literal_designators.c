// Compound literals with designated initializers
void putchar(int c);

typedef struct {
    int a;
    int b;
    int c;
    int d;
} Quad;

typedef struct {
    int x;
    int y;
    int z;
} Vec3;

int main() {
    // Test 1: Array with designated initializers
    int *arr = (int[5]){[0] = 10, [2] = 20, [4] = 30};
    if (arr[0] == 10 && arr[1] == 0 && arr[2] == 20 && 
        arr[3] == 0 && arr[4] == 30) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 2: Struct with designated initializers
    Quad q = (Quad){.b = 2, .d = 4, .a = 1};
    if (q.a == 1 && q.b == 2 && q.c == 0 && q.d == 4) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 3: Mixed designated and positional initializers
    Vec3 v = (Vec3){.x = 5, 6, .z = 7};
    if (v.x == 5 && v.y == 6 && v.z == 7) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 4: Array with range designator (GNU extension, may not work)
    // Fallback to explicit index designators
    int *range = (int[4]){[1] = 100, [2] = 100, [3] = 100};
    if (range[0] == 0 && range[1] == 100 && 
        range[2] == 100 && range[3] == 100) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}