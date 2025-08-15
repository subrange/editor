void putchar(int c);

int main() {
    char str[] = {'A', 'B', '\0'};
    char *p = str;
    
    putchar(*p);     // Should print 'A'
    p = p + 1;       // Increment pointer
    putchar(*p);     // Should print 'B'
    putchar('\n');
    
    return 0;
}