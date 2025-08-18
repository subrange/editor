typedef struct { 
    int x; 
} S; 

S test() { 
    S s; 
    s.x = 1; 
    return s; 
}

int main() {
    S result = test();
    if (result.x == 1) {
        putchar('Y');
    } else {
        putchar('N');
    }
    putchar('\n');
    return 0;
}

void putchar(int c);