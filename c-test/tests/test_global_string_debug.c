void putchar(int c);
void puts(char *s);

// Test global string initialization
char global_str1[] = "HELLO";
char global_str2[] = {'W', 'O', 'R', 'L', 'D', '\0'};
char *global_ptr = "POINTER";

int main() {
    puts("Testing global strings:");
    
    // Test 1: String literal initialization
    puts("T1: global_str1:");
    puts(global_str1);
    
    // Test 2: Explicit array initialization
    puts("T2: global_str2:");
    puts(global_str2);
    
    // Test 3: Pointer to string literal
    puts("T3: global_ptr:");
    puts(global_ptr);
    
    // Test 4: Direct character access
    puts("T4: Char by char:");
    putchar(global_str1[0]);
    putchar(global_str1[1]);
    putchar(global_str1[2]);
    putchar(global_str1[3]);
    putchar(global_str1[4]);
    putchar('\n');
    
    // Test 5: Check if it's null terminated
    puts("T5: Check null:");
    if (global_str1[5] == 0) {
        puts("YES - null terminated");
    } else {
        puts("NO - not null terminated!");
    }
    
    return 0;
}