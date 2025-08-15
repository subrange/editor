void putchar(int c);

int main() {
    int data = 0x41;  // 'A'
    char* p = (char*)&data;
    
    // Should print A
    putchar(*p);
    
    putchar('\n');
    return 0;
}