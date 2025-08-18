// Debug array parsing
void putchar(int c);

int main() {
    int arr[3];       // Array { element_type: Int, size: 3 }
    int mat[2][3];    // Should be: Array { element_type: Array { element_type: Int, size: 3 }, size: 2 }
    
    putchar('Y');
    return 0;
}