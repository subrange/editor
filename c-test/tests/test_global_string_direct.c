void putchar(int c);

// Test global string initialization without using __ld8
char greeting[] = "Hello";

int main() {
    // Access the string as 16-bit words and extract bytes manually
    // This avoids using the buggy __ld8 function
    unsigned short* words = (unsigned short*)greeting;
    
    // Word 0 contains 'H' and 'e'
    unsigned short w0 = words[0];
    putchar(w0 & 0xFF);        // 'H'
    putchar((w0 >> 8) & 0xFF); // 'e'
    
    // Word 1 contains 'l' and 'l'
    unsigned short w1 = words[1];
    putchar(w1 & 0xFF);        // 'l'
    putchar((w1 >> 8) & 0xFF); // 'l'
    
    // Word 2 contains 'o' and '\0'
    unsigned short w2 = words[2];
    putchar(w2 & 0xFF);        // 'o'
    
    putchar('\n');
    return 0;
}