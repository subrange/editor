// Simpler test for puts function
void putchar(int c);

int puts(char *str) {
    putchar(str[0]);
    putchar(str[1]);
    putchar(str[2]);
    putchar('\n');
    return 3;
}

int main() {
    char msg[] = {'A', 'B', 'C', '\0'};
    puts(msg);
    return 0;
}