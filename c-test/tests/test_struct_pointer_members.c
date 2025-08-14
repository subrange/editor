// Test structs with pointer members
void putchar(int c);

struct Node {
    int value;
    int* ptr;
};

int main() {
    int data1 = 42;
    int data2 = 99;
    struct Node node;
    
    // Initialize struct with pointer member
    node.value = 10;
    node.ptr = &data1;
    
    // Test regular field
    if (node.value == 10) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    // Test dereferencing pointer member
    if (*node.ptr == 42) {
        putchar('2');
    } else {
        putchar('N');
    }
    
    // Modify through pointer member
    *node.ptr = 55;
    if (data1 == 55) {
        putchar('3');
    } else {
        putchar('N');
    }
    
    // Change pointer to point elsewhere
    node.ptr = &data2;
    if (*node.ptr == 99) {
        putchar('4');
    } else {
        putchar('N');
    }
    
    // Test pointer member with arrow operator
    struct Node* node_ptr = &node;
    if (node_ptr->value == 10) {
        putchar('5');
    } else {
        putchar('N');
    }
    
    if (*node_ptr->ptr == 99) {
        putchar('6');
    } else {
        putchar('N');
    }
    
    // Modify through arrow operator
    node_ptr->value = 20;
    if (node.value == 20) {
        putchar('7');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}