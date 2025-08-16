// Test PHI nodes with fat pointers
// This verifies that conditional pointer assignments work correctly

void putchar(int c);

int global_x = 10;

int main() {
    int local_y = 20;
    int *ptr;
    
    // Test 1: Simple conditional pointer assignment
    if (global_x > 5) {
        ptr = &global_x;  // Global bank
    } else {
        ptr = &local_y;   // Stack bank
    }
    // At this point, ptr has Mixed provenance in the old system
    // With fat pointers, the bank is carried through the PHI
    
    int val = *ptr;
    if (val == 10) {
        putchar('1');  // Test passes
    } else {
        putchar('N');
    }
    
    // Test 2: Opposite condition
    if (global_x < 5) {
        ptr = &global_x;  // Global bank
    } else {
        ptr = &local_y;   // Stack bank  
    }
    
    val = *ptr;
    if (val == 20) {
        putchar('2');  // Test passes
    } else {
        putchar('N');
    }
    
    // Test 3: Loop with changing pointer
    int i;
    for (i = 0; i < 2; i = i + 1) {
        if (i == 0) {
            ptr = &global_x;
        } else {
            ptr = &local_y;
        }
        
        val = *ptr;
        if ((i == 0 && val == 10) || (i == 1 && val == 20)) {
            putchar('3' + i);  // Should print '3' then '4'
        } else {
            putchar('N');
        }
    }
    
    putchar('\n');
    return 0;
}