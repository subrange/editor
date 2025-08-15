// Test inline assembly with multiline strings
void putchar(int c);

int main() {
    // Using adjacent string concatenation for readable multiline assembly
    asm(
        "LI T0, 72; "     // 'H'
        "STORE T0, R0, R0; "
        "LI T0, 101; "    // 'e'
        "STORE T0, R0, R0; "
        "LI T0, 108; "    // 'l'
        "STORE T0, R0, R0; "
        "LI T0, 108; "    // 'l'
        "STORE T0, R0, R0; "
        "LI T0, 111; "    // 'o'
        "STORE T0, R0, R0; "
        "LI T0, 10; "     // '\n'
        "STORE T0, R0, R0"
    );
    
    return 0;
}