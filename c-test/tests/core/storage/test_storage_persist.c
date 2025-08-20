#include <stdio.h>
#include <mmio.h>

// This test writes a counter that increments each time it runs
// To test: run it multiple times and see the counter increase

void print_number(unsigned short n) {
    // Simple number printing using putchar
    if (n >= 10000) putchar('0' + (n / 10000) % 10);
    if (n >= 1000) putchar('0' + (n / 1000) % 10);
    if (n >= 100) putchar('0' + (n / 100) % 10);
    if (n >= 10) putchar('0' + (n / 10) % 10);
    putchar('0' + n % 10);
}

int main() {
    // Use block 99 for our persistent counter
    unsigned short counter = storage_read_at(99, 0);
    
    // If it's 0 (uninitialized), start at 1000
    if (counter == 0) {
        counter = 1000;
    }
    
    // Print current value
    puts("Counter: ");
    print_number(counter);
    putchar('\n');
    
    // Increment and save
    counter++;
    storage_write_at(99, 0, counter);
    
    // Commit to make it persistent
    storage_commit();
    
    return 0;
}