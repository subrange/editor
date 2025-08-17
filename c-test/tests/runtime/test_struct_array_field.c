// Test arrays as struct fields
#include <stdio.h>

struct Point {
    int coords[2];  // Fixed size array in struct
};

struct Buffer {
    unsigned char data[5];  // Array field
    int size;
};

int main() {
    // Struct with array field initialization
    struct Point p = {{3, 4}};
    
    // Struct with array field, individual initialization
    struct Buffer buf;
    buf.data[0] = 1;
    buf.data[1] = 2;
    buf.data[2] = 3;
    buf.data[3] = 4;
    buf.data[4] = 5;
    buf.size = 5;
    
    // Test Point struct
    putchar('0' + p.coords[0]);
    putchar('0' + p.coords[1]);
    putchar('\n');
    
    // Test Buffer struct
    for (int i = 0; i < buf.size; i++) {
        putchar('0' + buf.data[i]);
    }
    putchar('\n');
    
    // Dynamic access to struct array field
    for (int i = 0; i < 2; i++) {
        putchar('0' + p.coords[i] * 2);
    }
    putchar('\n');
    
    return 0;
}