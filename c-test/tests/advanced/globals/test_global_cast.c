void putchar(int c);

// Test that global string packing works correctly
// We'll cast through void* to int* to test the packed data
char greeting[] = "Hi";

int main() {
    // Cast &greeting[0] to void* first, then to int*
    void* vptr = (void*)&greeting[0];
    int* words = (int*)vptr;
    
    // Get first word - should be 0x6948 ('i' in high byte, 'H' in low byte)
    int w0 = words[0];
    
    // Extract and print bytes manually
    putchar(w0 & 0xFF);        // Should print 'H' (72)
    putchar((w0 >> 8) & 0xFF); // Should print 'i' (105)
    putchar('\n');
    
    return 0;
}