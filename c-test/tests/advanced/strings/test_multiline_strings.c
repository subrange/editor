// Test multiline string support
void putchar(int c);
int puts(char *str);

int main() {
    // Test 1: Multiline string with actual newlines  
    char* msg1 = "Hello
World
Test";
    
    // Test 2: Adjacent string concatenation
    char* msg2 = "This " "is " "concatenated";
    
    // Test 3: Multiline assembly with semicolons as separators
    asm("LI T0, 72; STORE T0, R0, R0; LI T0, 105; STORE T0, R0, R0");
    
    // Test 4: Multiline assembly with adjacent strings and semicolons
    asm("LI T1, 10; "
        "LI T2, 20; "  
        "ADD T3, T1, T2; "
        "STORE T3, R0, R0; ");

    // Test 5: Line continuation with backslash
    char* msg3 = "This is a \
continued line";
    
    puts(msg3);
    
    return 0;
}