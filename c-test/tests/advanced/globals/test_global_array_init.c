// Test global array initialization
void putchar(int c);

// Global array with initializer
int numbers[5] = {10, 20, 30, 40, 50};

int main() {
    // Test each element
    if (numbers[0] == 10) putchar('1'); else putchar('F');
    if (numbers[1] == 20) putchar('2'); else putchar('F');
    if (numbers[2] == 30) putchar('3'); else putchar('F');
    if (numbers[3] == 40) putchar('4'); else putchar('F');
    if (numbers[4] == 50) putchar('5'); else putchar('F');
    
    putchar('\n');
    return 0;
}