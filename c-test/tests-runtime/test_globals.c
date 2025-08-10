// Test global variables
int global_x = 42;
int global_y = 100;
int global_uninitialized;

void putchar(int c);

int main() {
    // Print global_x value (42 = '*')
    putchar(global_x);
    
    // Modify global
    global_x = 65; // 'A'
    putchar(global_x);
    
    // Use uninitialized global (should be 0)
    global_uninitialized = 10;  // '\n'
    putchar(global_uninitialized);
    
    return 0;
}