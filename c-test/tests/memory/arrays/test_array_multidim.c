// Test array indexing via GEP
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
    
    if (matrix[1][1] == 5) putchar('Y'); else putchar('N');

    int (*p)[3] = matrix;

    if (*(p[0] + 2) == 3) putchar('Y'); else putchar('N');

    int *q = &matrix[0][0];

    if (*(q + 3) == 4) putchar('Y'); else putchar('N');
    
    int (*r)[3] = &matrix[1];

    if (r[0][0] == 4) putchar('Y'); else putchar('N');
    
    return 0;
}