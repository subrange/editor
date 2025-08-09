// Runtime implementation of putchar
// Uses inline assembly to directly output to MMIO

void putchar(int c) {
    // Use inline assembly to store character to output port
    // R3 should contain the character value (first parameter)
    // STORE R3, R0, R0 outputs the character
    __asm__("STORE R3, R0, R0");
}