#include <stdio.h>

int main() {
    // Storage MMIO addresses
    unsigned short* store_block = (unsigned short*)17;
    unsigned short* store_addr = (unsigned short*)18;
    unsigned short* store_data = (unsigned short*)19;
    unsigned short* store_ctl = (unsigned short*)20;
    
    // Test 1: Write then read single value
    *store_block = 10;
    *store_addr = 100;
    *store_data = 5555;
    
    // Read it back immediately
    *store_block = 10;
    *store_addr = 100;
    if (*store_data == 5555) putchar('Y'); else putchar('N');
    
    // Test 2: Check auto-increment on write
    *store_block = 20;
    *store_addr = 0;
    *store_data = 100;  // Write to addr 0
    *store_data = 101;  // Should write to addr 1
    *store_data = 102;  // Should write to addr 2
    
    // Read back with manual addressing
    *store_addr = 0;
    if (*store_data == 100) putchar('Y'); else putchar('N');
    
    *store_addr = 1;
    if (*store_data == 101) putchar('Y'); else putchar('N');
    
    *store_addr = 2; 
    if (*store_data == 102) putchar('Y'); else putchar('N');
    
    // Test 3: Commit and persistence (write, commit, then read)
    *store_block = 30;
    *store_addr = 50;
    *store_data = 9999;
    
    // Commit the block
    *store_ctl = 4;  // COMMIT bit
    
    // Read back after commit
    *store_block = 30;
    *store_addr = 50;
    if (*store_data == 9999) putchar('Y'); else putchar('N');
    
    putchar('\n');
    
    return 0;
}