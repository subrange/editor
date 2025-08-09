// Test puts with global arrays
void putchar(int c);

// Global array 
char message[4] = {'A', 'B', 'C', '\0'};

int puts(char *str) {
    int i = 0;
    while (str[i] != '\0') {
        putchar(str[i]);
        i = i + 1;
    }
    putchar('\n');
    return i;
}

int main() {
    puts(message);
    return 0;
}