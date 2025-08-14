// Simpler post-increment test
void putchar(int c);

int main() {
    int x = 3;
    int y;
    
    y = x++;
    
    // x should be 4, y should be 3
    if (x == 4) putchar('A'); else putchar('N');
    if (y == 3) putchar('B'); else putchar('M');
    
    putchar('\n');
    return 0;
}