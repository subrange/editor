void putchar(int c);

int main() {
    int data = 0x4142;  // 'BA' in little-endian
    
    // Test extracting low byte (should be 'B' = 0x42)
    int low = data & 0xFF;
    putchar(low);
    
    // Test extracting high byte (should be 'A' = 0x41)  
    int high = (data >> 8) & 0xFF;
    putchar(high);
    
    putchar('\n');
    return 0;
}