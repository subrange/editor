// Test multidimensional arrays without pointer-to-array
void putchar(int c);

int main() {
    int matrix[2][3];
    
    // Initialize matrix
    matrix[0][0] = 1;
    matrix[0][1] = 2;
    matrix[0][2] = 3;
    matrix[1][0] = 4;
    matrix[1][1] = 5;
    matrix[1][2] = 6;
    
    // Test direct access
    if (matrix[0][0] == 1) putchar('Y'); else putchar('N');
    if (matrix[0][2] == 3) putchar('Y'); else putchar('N');
    if (matrix[1][0] == 4) putchar('Y'); else putchar('N');
    if (matrix[1][2] == 6) putchar('Y'); else putchar('N');
    
    return 0;
}