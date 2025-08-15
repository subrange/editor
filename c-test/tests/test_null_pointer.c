void putchar(int c);

int main() {
    int *p = 0;  // NULL pointer
    
    if (p == 0) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    return 0;
}
