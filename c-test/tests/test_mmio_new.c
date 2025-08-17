// Test new MMIO layout with TTY, RNG, and TEXT40 display

// MMIO addresses
#define TTY_OUT       (*((unsigned short*)0))
#define TTY_STATUS    (*((unsigned short*)1))
#define TTY_IN_POP    (*((unsigned short*)2))
#define TTY_IN_STATUS (*((unsigned short*)3))
#define RNG           (*((unsigned short*)4))
#define DISP_MODE     (*((unsigned short*)5))
#define DISP_STATUS   (*((unsigned short*)6))
#define DISP_CTL      (*((unsigned short*)7))
#define DISP_FLUSH    (*((unsigned short*)8))

// Display modes
#define DISP_OFF    0
#define DISP_TTY    1
#define DISP_TEXT40 2

// Display control bits
#define DISP_ENABLE (1 << 0)
#define DISP_CLEAR  (1 << 1)

// TEXT40 VRAM starts at word 32
#define VRAM ((unsigned short*)32)

void putchar(int c) {
    // Wait for TTY ready
    while (!(TTY_STATUS & 1)) {}
    TTY_OUT = c;
}

int main() {
    // Test 1: Basic TTY output
    putchar('T');
    putchar('T');
    putchar('Y');
    putchar(':');
    putchar('O');
    putchar('K');
    putchar('\n');
    
    // Test 2: Read RNG values
    unsigned short rng1 = RNG;
    unsigned short rng2 = RNG;
    unsigned short rng3 = RNG;
    
    // RNG values should be different
    if (rng1 != rng2 && rng2 != rng3) {
        putchar('R');
        putchar('N');
        putchar('G');
        putchar(':');
        putchar('O');
        putchar('K');
        putchar('\n');
    } else {
        putchar('R');
        putchar('N');
        putchar('G');
        putchar(':');
        putchar('N');
        putchar('O');
        putchar('\n');
    }
    
    // Test 3: TEXT40 display mode
    DISP_MODE = DISP_TEXT40;
    DISP_CTL = DISP_ENABLE | DISP_CLEAR;  // Enable and clear
    
    // Write "HELLO" to VRAM at position 0
    VRAM[0] = 'H';
    VRAM[1] = 'E';
    VRAM[2] = 'L';
    VRAM[3] = 'L';
    VRAM[4] = 'O';
    
    // Write "WORLD" at position 40 (second line)
    VRAM[40] = 'W';
    VRAM[41] = 'O';
    VRAM[42] = 'R';
    VRAM[43] = 'L';
    VRAM[44] = 'D';
    
    // Flush display
    DISP_FLUSH = 1;
    
    // Check display status
    if (DISP_STATUS & 2) { // Check flush_done bit
        putchar('D');
        putchar('I');
        putchar('S');
        putchar('P');
        putchar(':');
        putchar('O');
        putchar('K');
        putchar('\n');
    }
    
    return 0;
}