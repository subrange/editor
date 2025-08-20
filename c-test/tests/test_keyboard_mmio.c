// Test keyboard MMIO functionality (non-interactive)
#include <stdio.h>
#include <mmio.h>
#include <mmio_constants.h>

int main() {
    // Test that we can read keyboard state addresses without crashing
    // In non-TEXT40 mode, these should all return 0
    
    // Read all keyboard MMIO addresses
    int up = key_up_pressed();
    int down = key_down_pressed();
    int left = key_left_pressed();
    int right = key_right_pressed();
    int z = key_z_pressed();
    int x = key_x_pressed();
    
    // In non-TEXT40 mode, all should be 0
    if (up == 0 && down == 0 && left == 0 && right == 0 && z == 0 && x == 0) {
        putchar('Y'); // Yes, working correctly
    } else {
        putchar('N'); // No, something wrong
    }
    putchar('\n');
    
    // Now test in TEXT40 mode (briefly)
    display_set_mode(DISP_MODE_TEXT40);
    display_enable();
    
    // Read keyboard states again (still should be 0 since no keys pressed)
    up = key_up_pressed();
    down = key_down_pressed();
    
    if (up == 0 && down == 0) {
        putchar('Y'); // Yes, still working
    } else {
        putchar('N'); // No, unexpected values
    }
    putchar('\n');
    
    // Return to normal mode
    display_set_mode(DISP_MODE_OFF);
    
    return 0;
}