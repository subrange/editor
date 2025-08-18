// Test accessing int field through pointer to struct
void putchar(int c);

struct Data {
    int a;
    int b;
};

int main() {
    struct Data data;
    struct Data* ptr;
    
    data.a = 5;
    data.b = 10;
    ptr = &data;
    
    // Test accessing int field through pointer
    if (ptr->b == 10) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}