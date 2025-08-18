// Test array dimension ordering
void putchar(int c);

int main() {
    int matrix[2][3]; // Should be: Array of 2 (Array of 3 int)
    
    // matrix[0] should be int[3]
    // matrix[1] should be int[3]
    
    matrix[0][0] = 1;
    matrix[0][1] = 2;
    matrix[0][2] = 3;
    matrix[1][0] = 4;
    matrix[1][1] = 5;
    matrix[1][2] = 6;
    
    // Test direct indexing
    if (matrix[0][0] == 1) putchar('Y'); else putchar('N');
    if (matrix[0][2] == 3) putchar('Y'); else putchar('N');
    if (matrix[1][0] == 4) putchar('Y'); else putchar('N');
    if (matrix[1][2] == 6) putchar('Y'); else putchar('N');
    
    return 0;
}