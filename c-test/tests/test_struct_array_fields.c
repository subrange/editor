// Test structs with array members
void putchar(int c);

struct Buffer {
    int size;
    int data[5];
    int checksum;
};

int main() {
    struct Buffer buf;
    int i;
    
    // Initialize struct
    buf.size = 5;
    buf.checksum = 0;
    
    // Initialize array in struct
    for (i = 0; i < 5; i = i + 1) {
        buf.data[i] = i * 10;
    }
    
    // Test size field
    if (buf.size == 5) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    // Test array access
    if (buf.data[0] == 0) {
        putchar('2');
    } else {
        putchar('N');
    }
    
    if (buf.data[2] == 20) {
        putchar('3');
    } else {
        putchar('N');
    }
    
    if (buf.data[4] == 40) {
        putchar('4');
    } else {
        putchar('N');
    }
    
    // Modify array in struct
    buf.data[1] = 99;
    if (buf.data[1] == 99) {
        putchar('5');
    } else {
        putchar('N');
    }
    
    // Calculate checksum
    for (i = 0; i < 5; i = i + 1) {
        buf.checksum = buf.checksum + buf.data[i];
    }
    
    // 0 + 99 + 20 + 30 + 40 = 189
    if (buf.checksum == 189) {
        putchar('6');
    } else {
        putchar('N');
    }
    
    // Test through pointer
    struct Buffer* ptr = &buf;
    if (ptr->data[2] == 20) {
        putchar('7');
    } else {
        putchar('N');
    }
    
    ptr->data[3] = 100;
    if (buf.data[3] == 100) {
        putchar('8');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}