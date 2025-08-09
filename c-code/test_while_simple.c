// Simple while loop test
void putchar(int c);

int main() {
    int i = 0;
    
    putchar('S');
    putchar(':');
    
    while (i < 3) {
        putchar('0' + i);
        i = i + 1;
    }
    
    putchar(10);
    putchar('E');
    putchar(10);
    
    return 0;
}