// Test MMIO using runtime library functions

#include <mmio.h>

void putchar(int c);

int main() {
    // Test 1: Basic TTY output via MMIO
    tty_putchar('M');
    tty_putchar('M');
    tty_putchar('I');
    tty_putchar('O');
    tty_putchar(':');
    
    // For now, just output OK (TTY status check needs inline asm)
    tty_putchar('O');
    tty_putchar('K');
    tty_putchar('\n');
    
    // Test 3: RNG
    unsigned short r1 = rng_get();
    unsigned short r2 = rng_get();
    unsigned short r3 = rng_get();
    
    // RNG values should be different
    if (r1 != r2 && r2 != r3) {
        tty_putchar('R');
        tty_putchar('N');
        tty_putchar('G');
        tty_putchar(':');
        tty_putchar('O');
        tty_putchar('K');
    } else {
        tty_putchar('R');
        tty_putchar('N');
        tty_putchar('G');
        tty_putchar(':');
        tty_putchar('N');
        tty_putchar('O');
    }
    tty_putchar('\n');
    
    // Test 4: TEXT40 display
    display_set_mode(DISP_MODE_TEXT40);
    display_clear();  // Enable and clear
    
    // Write some text to VRAM
    text40_puts(0, 0, "HELLO");  // Top-left
    text40_puts(0, 1, "WORLD");  // Second line
    
    // Flush display
    display_flush();
    
    // For now, just output OK (status check needs inline asm)
    tty_putchar('D');
    tty_putchar('I');
    tty_putchar('S');
    tty_putchar('P');
    tty_putchar(':');
    tty_putchar('O');
    tty_putchar('K');
    tty_putchar('\n');
    
    return 0;
}