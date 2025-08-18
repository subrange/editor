// Simple test for struct pointer field
void putchar(int c);

struct Data {
    int value;
};

struct Container {
    struct Data* ptr;
};

int main() {
    struct Data data;
    struct Container cont;
    
    data.value = 42;
    cont.ptr = &data;
    
    // Direct check
    if (cont.ptr->value == 42) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}