// Test simple array to pointer decay for multidimensional arrays
void putchar(int c);

int main() {
    int matrix[2][3];
    matrix[0][0] = 1;
    matrix[0][1] = 2;
    matrix[0][2] = 3;
    matrix[1][0] = 4;
    matrix[1][1] = 5;
    matrix[1][2] = 6;
    
    // matrix should decay to int(*)[3]
    int (*p)[3] = matrix;
    
    // Test that p points to the first row
    if (p[0][0] == 1) putchar('Y'); else putchar('N');
    if (p[0][1] == 2) putchar('Y'); else putchar('N');
    if (p[1][0] == 4) putchar('Y'); else putchar('N');
    
    return 0;
}