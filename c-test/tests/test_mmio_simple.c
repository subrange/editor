// Simple test of new MMIO functionality

void putchar(int c);

// Direct memory access functions
unsigned short read_mem(unsigned short addr) {
    unsigned short* ptr = (unsigned short*)(int)addr;
    return *ptr;
}

void write_mem(unsigned short addr, unsigned short value) {
    unsigned short* ptr = (unsigned short*)(int)addr;
    *ptr = value;
}

int main() {
    // Test basic TTY output using address 0
    write_mem(0, 'H');
    write_mem(0, 'i');
    write_mem(0, '\n');
    
    // Test RNG at address 4
    unsigned short rng1 = read_mem(4);
    unsigned short rng2 = read_mem(4);
    
    // RNG values should be different
    if (rng1 != rng2) {
        write_mem(0, 'O');
        write_mem(0, 'K');
    } else {
        write_mem(0, 'N');
        write_mem(0, 'O');
    }
    write_mem(0, '\n');
    
    return 0;
}