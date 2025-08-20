#include <stdio.h>

int main() {
    // Storage MMIO addresses
    unsigned short* store_block = (unsigned short*)17;
    unsigned short* store_addr = (unsigned short*)18;
    unsigned short* store_data = (unsigned short*)19;
    unsigned short* store_ctl = (unsigned short*)20;
    
    // Write some data to block 1
    *store_block = 1;  // Select block 1
    *store_addr = 0;   // Start at word 0
    
    // Write values 1000-1004
    *store_data = 1000;
    *store_data = 1001;
    *store_data = 1002;
    *store_data = 1003;
    *store_data = 1004;
    
    // Commit the block
    *store_ctl = 4;  // Bit 2 = COMMIT
    
    // Now read back the data
    *store_block = 1;  // Select block 1 again
    *store_addr = 0;   // Start at word 0
    
    // Read and verify
    if (*store_data == 1000) putchar('Y'); else putchar('N');
    if (*store_data == 1001) putchar('Y'); else putchar('N');
    if (*store_data == 1002) putchar('Y'); else putchar('N');
    if (*store_data == 1003) putchar('Y'); else putchar('N');
    if (*store_data == 1004) putchar('Y'); else putchar('N');
    
    putchar('\n');
    
    return 0;
}