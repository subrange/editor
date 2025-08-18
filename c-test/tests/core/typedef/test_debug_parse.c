// Debug test to understand type parsing
void putchar(int c);

int main() {
    int arr[3];
    int *p1 = arr; // Should work - arr decays to int*
    
    int matrix[2][3];
    int (*p2)[3]; // Declare pointer to array of 3 ints
    p2 = matrix; // This should work - matrix decays to int(*)[3]
    
    putchar('Y');
    return 0;
}