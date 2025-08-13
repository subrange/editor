// Simple test for global variables
int global_var = 42;

void putchar(int c);

int main() {
    if (global_var == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    return 0;
}