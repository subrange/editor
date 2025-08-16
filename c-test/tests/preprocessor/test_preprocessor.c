void putchar(int c);

#define HELLO 'H'
#define WORLD 'W'

int main() {
    putchar(HELLO);
    putchar('e');
    putchar('l');
    putchar('l');
    putchar('o');
    putchar(' ');
    putchar(WORLD);
    putchar('o');
    putchar('r');
    putchar('l');
    putchar('d');
    putchar('\n');
    
    #ifdef DEBUG
    putchar('D');
    #else
    putchar('R');
    #endif
    
    putchar('\n');
    return 0;
}