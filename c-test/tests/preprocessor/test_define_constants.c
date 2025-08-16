void putchar(int c);

#define ZERO 0
#define ONE 1
#define TWO 2
#define THREE 3

int main() {
    if (ZERO == 0) putchar('Y'); else putchar('N');
    if (ONE == 1) putchar('Y'); else putchar('N');
    if (TWO == 2) putchar('Y'); else putchar('N');
    if (THREE == 3) putchar('Y'); else putchar('N');
    putchar('\n');
    return 0;
}