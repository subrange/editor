// Minimal test case to debug the boolean logic issue
void putchar(int c);

int main() {
    int i = 0;
    int val = 20;
    
    // This should be: (0 == 0 && 20 == 10) || (0 == 1 && 20 == 20)
    // Which is: (true && false) || (false && true) 
    // Which is: false || false = false
    if ((i == 0 && val == 10) || (i == 1 && val == 20)) {
        putchar('F'); // Should not execute
    } else {
        putchar('5'); // Should execute
    }
    
    putchar('\n');
    return 0;
}