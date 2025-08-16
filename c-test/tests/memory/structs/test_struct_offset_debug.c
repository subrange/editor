// Debug struct offsets
void putchar(int c);

struct Outer {
    int x;      // offset 0
    int inner1; // offset 1 
    int inner2; // offset 2
    int y;      // offset 3
};

int main() {
    struct Outer obj;
    
    obj.x = 10;
    obj.inner1 = 20;
    obj.inner2 = 30;
    obj.y = 40;
    
    if (obj.x == 10) putchar('1'); else putchar('N');
    if (obj.inner1 == 20) putchar('2'); else putchar('N');
    if (obj.inner2 == 30) putchar('3'); else putchar('N');
    if (obj.y == 40) putchar('4'); else putchar('N');
    
    putchar('\n');
    return 0;
}