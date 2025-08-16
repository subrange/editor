void putchar(int c);

// Test nested typedef
typedef int int32;
typedef int32 my_int;

// Test typedef with multiple declarators
typedef int num, *numptr;

// Test typedef of array
typedef int arr10[10];

// Test typedef of pointer to array
typedef int (*arrptr)[5];

// Function pointers not yet supported - will add in future

int main() {
    // Test nested typedef
    my_int x = 42;
    if (x == 42) {
        putchar('1');
    } else {
        putchar('N');
    }
    
    // Test multiple declarators
    num n = 5;
    numptr np = &n;
    if (*np == 5) {
        putchar('2');
    } else {
        putchar('N');
    }
    
    // Test array typedef
    arr10 array;
    array[0] = 10;
    array[9] = 20;
    if (array[0] + array[9] == 30) {
        putchar('3');
    } else {
        putchar('N');
    }
    
    // Test pointer to array typedef
    int real_array[5] = {1, 2, 3, 4, 5};
    arrptr ap = &real_array;
    if ((*ap)[2] == 3) {
        putchar('4');
    } else {
        putchar('N');
    }
    
    putchar('\n');
    return 0;
}