// Debug while loop
void putchar(int c);

int main() {
    putchar('A');  // Before loop
    
    int i = 0;
    
    putchar('B');  // After init
    
    while (i < 3) {
        putchar('L');  // In loop
        putchar('0' + i);
        i = i + 1;
    }
    
    putchar('C');  // After loop
    putchar(10);
    
    return 0;
}