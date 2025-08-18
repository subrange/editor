// Verify parser fix - this should pass semantic analysis
void putchar(int c);

int main() {
    int matrix[2][3];
    
    // This should work now - matrix decays to int(*)[3]
    int (*p)[3] = matrix;
    
    // This should also work - taking address of first row
    int (*q)[3] = &matrix[0];
    
    // Can't test actual functionality due to backend limitations
    // but at least it should compile past semantic analysis
    
    putchar('Y'); // Just output something
    return 0;
}