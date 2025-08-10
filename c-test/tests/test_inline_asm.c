// Test inline assembly support
// This test verifies that we can use inline assembly to directly emit Ripple assembly instructions

int main() {
    // Use inline assembly to load 'Y' into R3 and store to output
    __asm__("LI R3, 89");      // 89 is ASCII for 'Y'
    __asm__("STORE R3, R0, R0"); // Output the character
    
    // Use inline assembly to output a newline
    asm("LI R3, 10");           // 10 is newline
    asm("STORE R3, R0, R0");    // Output newline
    
    return 0;
}