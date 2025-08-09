// Test program using runtime library functions
extern void putchar(int c);
extern int puts(char *s);
extern char *memset(char *s, int c, int n);
extern char *memcpy(char *dest, char *src, int n);

int main() {
    // Test puts
    puts("Hello");
    
    // Test direct putchar
    putchar('!');
    putchar('\n');
    
    // Test memset and memcpy
    char buffer[10];
    memset(buffer, 'A', 5);
    buffer[5] = 0;  // null terminate
    
    // Output the buffer
    for (int i = 0; i < 5; i = i + 1) {
        putchar(buffer[i]);
    }
    putchar('\n');
    
    return 0;
}