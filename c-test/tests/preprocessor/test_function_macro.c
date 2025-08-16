void putchar(int c);

#define MAX(a, b) ((a) > (b) ? (a) : (b))
#define MIN(a, b) ((a) < (b) ? (a) : (b))
#define ABS(x) ((x) < 0 ? -(x) : (x))

int main() {
    int a = 5;
    int b = 10;
    
    if (MAX(a, b) == 10) putchar('Y'); else putchar('N');
    if (MIN(a, b) == 5) putchar('Y'); else putchar('N');
    if (ABS(-3) == 3) putchar('Y'); else putchar('N');
    if (ABS(3) == 3) putchar('Y'); else putchar('N');
    
    putchar('\n');
    return 0;
}