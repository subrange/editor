// Test struct field offsets and sizes
void putchar(int c);

struct Mixed {
    int a;        // offset 0, size 1
    char b;       // offset 1, size 1  
    int c;        // offset 2, size 1
    long d;       // offset 3, size 2
    int e;        // offset 5, size 1
    int arr[3];   // offset 6, size 3
    int f;        // offset 9, size 1
};

int main() {
    struct Mixed m;
    struct Mixed* ptr = &m;
    
    // Initialize all fields
    m.a = 10;
    m.b = 20;
    m.c = 30;
    m.d = 40;
    m.e = 50;
    m.arr[0] = 60;
    m.arr[1] = 70;
    m.arr[2] = 80;
    m.f = 90;
    
    // Test that fields don't overlap (each field keeps its value)
    if (m.a == 10) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    if (m.b == 20) {
        putchar('2');
    } else {
        putchar('N');
    }
    
    if (m.c == 30) {
        putchar('3');
    } else {
        putchar('N');
    }
    
    if (m.d == 40) {
        putchar('4');
    } else {
        putchar('N');
    }
    
    if (m.e == 50) {
        putchar('5');
    } else {
        putchar('N');
    }
    
    if (m.arr[0] == 60 && m.arr[1] == 70 && m.arr[2] == 80) {
        putchar('6');
    } else {
        putchar('N');
    }
    
    if (m.f == 90) {
        putchar('7');
    } else {
        putchar('N');
    }
    
    // Test through pointer to ensure offsets work correctly
    ptr->a = 100;
    ptr->f = 200;
    
    if (m.a == 100 && m.f == 200) {
        putchar('8');
    } else {
        putchar('N');
    }
    
    // Verify middle fields unchanged
    if (m.c == 30 && m.e == 50) {
        putchar('9');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}