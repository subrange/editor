// Returns byte in low 8 bits of Rv0.
void __ld8(unsigned short bank, unsigned short byte_addr) {
    // A0=bank, A1=byte_addr, Rv0=return, T0/T1 scratch
    asm("SRLI T0, A1, 1");          // word_addr = byte_addr >> 1
    asm("LOAD T1, A0, T0");         // word = mem[bank][word_addr]
    asm("ANDI T0, A1, 1");          // half = byte_addr & 1
    asm("BNE  T0, R0, __ld8_hi");   // if half!=0 -> high byte

    // low byte path
    asm("ANDI Rv0, T1, 0x00FF");    // Rv0 = word & 0xFF
    asm("BEQ  R0, R0, __ld8_done");

    // high byte path
    asm("__ld8_hi:");
    asm("SRLI Rv0, T1, 8");         // Rv0 = (word >> 8) & 0xFF
    asm("ANDI Rv0, Rv0, 0x00FF");   // No return in a function, because we fill it here

    asm("__ld8_done:");
}

// Stores low 8 bits of value to the addressed byte; Rv0 unused.
void __st8(unsigned short bank, unsigned short byte_addr, unsigned short value) {
    // A0=bank, A1=byte_addr, A2=value, T0/T1 scratch
    asm("SRLI T0, A1, 1");          // word_addr = byte_addr >> 1
    asm("LOAD T1, A0, T0");         // old = mem[bank][word_addr]
    asm("ANDI T0, A1, 1");          // half = byte_addr & 1
    asm("BNE  T0, R0, __st8_hi");   // if half!=0 -> high byte

    // low byte path: new = (old & 0xFF00) | (value & 0x00FF)
    asm("ANDI T1, T1, 0xFF00");     // keep old high byte
    asm("ANDI T0, A2, 0x00FF");     // val8
    asm("OR   T1, T1, T0");
    asm("BEQ  R0, R0, __st8_store");

    // high byte path: new = (old & 0x00FF) | ((value & 0xFF)<<8)
    asm("__st8_hi:");
    asm("ANDI T1, T1, 0x00FF");     // keep old low byte
    asm("ANDI T0, A2, 0x00FF");     // val8
    asm("SLLI T0, T0, 8");
    asm("OR   T1, T1, T0");

    // write-back
    asm("__st8_store:");
    asm("SRLI T0, A1, 1");          // recompute word_addr
    asm("STORE T1, A0, T0");
}