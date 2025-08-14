void putchar(int c);

void test_alloca(char ch) {
    char local_ch = ch;  // This creates an alloca
    putchar(local_ch);
    putchar('\n');
}

void test_alloca_complex(char *arr, int *len, int pos, char ch) {
    char local_ch = ch;  // This creates an alloca
    putchar(local_ch);
    putchar('\n');
}

int main() {
    test_alloca('Y');
    
    char list[10];
    int len = 3;
    test_alloca_complex(list, &len, 1, 'Z');
    
    return 0;
}