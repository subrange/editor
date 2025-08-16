// Test pointer difference with NULL pointers
void putchar(int c);

int main() {
    int* p = (int*)0;   // NULL pointer
    int* s = (int*)10;  // Non-NULL pointer
    int diff = s - p;   // 10 - 0 = 10
    
    // Debug: print the actual diff value
    if (diff == 10) {
        putchar('Y');
    } else if (diff == 0) {
        putchar('0');
    } else if (diff == 5) {
        putchar('5');
    } else {
        putchar('?');
    }
    putchar('\n');
    return 0;
}