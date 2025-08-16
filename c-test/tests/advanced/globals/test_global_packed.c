void putchar(int c);

// Test that global string packing works correctly
// We'll access it as words to bypass the buggy __ld8
char greeting[] = "Hi";

int main() {
    // Access greeting as a word pointer to test packing
    // "Hi\0" should be packed as:
    // Word 0: 0x6948 ('i' in high byte, 'H' in low byte)
    // Word 1: 0x0000 (null terminators)
    
    // Cast to int pointer (which our compiler treats as 16-bit)
    int* words = (int*)greeting;
    
    // Get first word
    int w0 = words[0];
    
    // Extract and print bytes manually
    putchar(w0 & 0xFF);        // Should print 'H' (72)
    putchar((w0 >> 8) & 0xFF); // Should print 'i' (105)
    putchar('\n');
    
    return 0;
}