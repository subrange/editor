// Test program without _start (uses crt0.asm)
// Compile this without _start and link with crt0.pobj

int main() {
    // Direct MMIO output since putchar is special-cased
    __asm__("LI R3, 78");      // 'N'
    __asm__("STORE R3, R0, R0");
    __asm__("LI R3, 79");      // 'O'
    __asm__("STORE R3, R0, R0");
    __asm__("LI R3, 10");      // '\n'
    __asm__("STORE R3, R0, R0");
    
    return 0;
}