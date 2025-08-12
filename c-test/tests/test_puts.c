void putchar(int c);
void puts(char* c);

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
