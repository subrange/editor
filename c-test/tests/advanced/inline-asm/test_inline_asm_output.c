void putchar(int c);

int main() {
    int result = 0;
    
    // Test basic output operand
    __asm__("li %0, 42" : "=r"(result));
    
    if (result == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    // Test read-write operand
    int value = 10;
    __asm__("addi %0, %0, 5" : "+r"(value));
    
    if (value == 15) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    
    return 0;
}