// Simpler test for puts function
void putchar(int c);

int puts(char *str);

int main() {
    char msg[] = {'A', 'B', 'C', '\0'};
    puts(msg);
    return 0;
}