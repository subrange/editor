void putchar(int c);

int main() {
    char *msg = "Please, ASCII art, my Ripple!\n";
    
    // Print string character by character
    putchar(msg[0]);  // 'H'
    putchar(msg[1]);  // 'i'
    putchar(msg[2]);  // '!'
    putchar(msg[3]);  // '\n'
    
    return 0;
}