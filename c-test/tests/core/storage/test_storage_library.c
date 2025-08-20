#include <stdio.h>
#include <mmio.h>

int main() {
    // Test 1: Write and read a single value
    storage_write_at(10, 100, 5555);
    
    // Read it back immediately
    if (storage_read_at(10, 100) == 5555) 
        putchar('Y'); 
    else 
        putchar('N');
    
    // Test 2: Write multiple values with auto-increment
    unsigned short test_data[] = {100, 101, 102, 103, 104};
    storage_write_buffer(20, 0, test_data, 5);
    
    // Read back
    unsigned short read_data[5];
    storage_read_buffer(20, 0, read_data, 5);
    
    // Verify
    for (int i = 0; i < 5; i++) {
        if (read_data[i] == test_data[i])
            putchar('Y');
        else
            putchar('N');
    }
    
    // Test 3: Commit and verify persistence
    storage_write_at(30, 50, 9999);
    storage_commit();
    
    // Read back after commit
    if (storage_read_at(30, 50) == 9999)
        putchar('Y');
    else
        putchar('N');
    
    putchar('\n');
    
    return 0;
}