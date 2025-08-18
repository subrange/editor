// Test pointer field at offset 1
void putchar(int c);

struct Data {
    int value;
};

struct Container {
    int dummy;  // Force ptr to be at offset 1
    struct Data* ptr;
};

int main() {
    struct Data data;
    struct Container cont;
    
    data.value = 42;
    cont.dummy = 99;
    cont.ptr = &data;
    
    // Check
    if (cont.ptr->value == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}