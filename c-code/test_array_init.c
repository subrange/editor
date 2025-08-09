// Test array initializers
void putchar(int c);

int main() {
    // Test 1: Simple int array initializer
    int primes[5] = {2, 3, 5, 7, 11};
    
    // Test 2: Char array with initializer list
    char vowels[] = {'a', 'e', 'i', 'o', 'u', '\0'};
    
    // Test 3: String literal as char array initializer
    char hello[] = "Hello";
    
    // Print first prime
    if (primes[0] == 2) {
        putchar('1');  // Test 1 passed
    } else {
        putchar('F');  // Test 1 failed
    }
    
    // Print third prime
    if (primes[2] == 5) {
        putchar('2');  // Test 2 passed
    } else {
        putchar('F');  // Test 2 failed
    }
    
    // Print first vowel
    if (vowels[0] == 'a') {
        putchar('3');  // Test 3 passed
    } else {
        putchar('F');  // Test 3 failed
    }
    
    // Print from string literal initialized array
    if (hello[1] == 'e') {
        putchar('4');  // Test 4 passed
    } else {
        putchar('F');  // Test 4 failed
    }
    
    putchar('\n');
    
    return 0;
}