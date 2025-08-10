// Test implementing puts function using arrays and loops
void putchar(int c);

// Our own puts implementation
int puts(char *str) {
    int i = 0;
    while (str[i] != '\0') {
        putchar(str[i]);
        i = i + 1;
    }
    putchar('\n');  // puts adds a newline
    return i;  // Return number of characters written (excluding newline)
}

int main() {
    // Test 1: Simple string
    char hello[] = "Hello";
    puts(hello);
    
    // Test 2: Another string with array initializer
    char world[] = {'W', 'o', 'r', 'l', 'd', '\0'};
    puts(world);
    
    // Test 3: String literal (tests pointer parameter)
    puts("Test!");
    
    // Test 4: Empty string
    char empty[] = {'\0'};
    puts(empty);
    
    // Test 5: Single character
    char single[] = {'X', '\0'};
    puts(single);
    
    return 0;
}