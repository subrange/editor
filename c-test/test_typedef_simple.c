// Test basic typedef support
typedef int myint;

int main() {
    myint x = 42;
    
    if (x == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    return 0;
}