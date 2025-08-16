// Debug array indexing
void putchar(int c);

int main() {
    int arr[10];
    
    // Initialize manually  
    arr[0] = 65; // 'A'
    arr[1] = 66; // 'B'
    arr[2] = 67; // 'C'
    arr[3] = 68; // 'D'
    arr[4] = 69; // 'E'
    arr[5] = 70; // 'F'
    arr[6] = 71; // 'G'
    arr[7] = 72; // 'H'
    arr[8] = 73; // 'I'
    arr[9] = 74; // 'J'
    
    // Test direct access
    putchar(arr[0]); // Should print 'A'
    putchar(arr[6]); // Should print 'G'
    putchar(arr[9]); // Should print 'J'
    putchar('\n');
    
    // Test with variable
    int idx = 6;
    putchar(arr[idx]); // Should print 'G'
    putchar('\n');
    
    return 0;
}