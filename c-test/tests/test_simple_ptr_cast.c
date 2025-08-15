void putchar(int c);

int main() {
    int data = 0x4142;
    char* p = (char*)&data;
    
    // Should print B (0x42)
    putchar(*p);
    
    // Should print A (0x41) 
    char* p2 = p + 1;
    putchar(*p2);
    
    putchar('\n');
    return 0;
}