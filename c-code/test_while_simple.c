// Simple while loop test
void putchar(int c);

int main() {
    int i = 0;
    int count = 0;
    
    // Count iterations
    while (i < 3) {
        count = count + 1;
        i = i + 1;
    }
    
    // Verify loop ran 3 times
    if (count == 3) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Verify final value of i
    if (i == 3) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    
    return 0;
}