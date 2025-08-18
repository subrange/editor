// Compound literal scope and lifetime tests
void putchar(int c);

typedef struct {
    int value;
} Item;

int* get_array_ptr() {
    // Compound literal has automatic storage duration
    return (int[]){1, 2, 3};  // Returns pointer to local
}

Item* get_valid_item() {
    static Item item = {0};
    item = (Item){100};
    return &item;
}

int main() {
    // Test 1: Compound literal in same scope
    int *p1 = (int[]){10, 20, 30};
    if (p1[0] == 10 && p1[1] == 20 && p1[2] == 30) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 2: Multiple compound literals
    int *p2 = (int[]){40};
    int *p3 = (int[]){50};
    if (*p2 == 40 && *p3 == 50) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    // Test 3: Compound literal in loop
    for (int i = 0; i < 2; i++) {
        int *arr = (int[]){i, i+1, i+2};
        if (arr[0] == i && arr[1] == i+1 && arr[2] == i+2) {
            putchar('Y');
        } else {
            putchar('N');
        }
    }
    
    // Test 4: Valid item from function
    Item *item = get_valid_item();
    if (item->value == 100) {
        putchar('Y');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}