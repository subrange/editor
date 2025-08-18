// Simple test for global pointer to string literal
void puts(char *s);

char *global_ptr = "TEST";

int main() {
    puts(global_ptr);
    return 0;
}