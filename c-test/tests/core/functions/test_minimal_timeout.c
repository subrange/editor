// Minimal test to reproduce the timeout issue
void putchar(int c);
void puts(char *s);

struct Point {
    int x;
    int y;
};

struct Complex {
    int id;
    struct Point position;
    char name[8];
    int* data_ptr;
    int flags;
};

int main() {
    // This works fine
    struct Complex c2;
    c2.id = 777;
    puts("Before");
    
    // Adding these lines causes timeout???
    struct Point p1;
    struct Point p2;
    puts("After");
    
    return 0;
}