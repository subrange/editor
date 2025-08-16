void putchar(int c);

/*
  DEMO:
  In Ripple VM, char takes one word (16 bits).
  Working with tightly packed char requires additional care.
*/

int main() {
    int data[] = {0x4142, 0x4344}; // 'ABCD' in little endian (0x42='B', 0x41='A', 0x44='D', 0x43='C')
    char* p = (char*)&data;
    
    // Should print B (0x42)
    putchar(*p);
    
    // Should print D (0x44)
    char* p2 = p + 1; // Moving to the next word
    putchar(*p2);

    // To access A and C, we need to move the pointer, and then use shift:
    putchar(*(p) >> 8);
    putchar(*(p2) >> 8); // Should print C (0x43)
    
    putchar('\n');
    return 0;
}