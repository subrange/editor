void putchar(int c);

void modify_array(char *arr, int index, char value) {
    arr[index] = value;
}

int main() {
    char data[5];
    data[0] = 'A';
    data[1] = 'B';
    data[2] = 'C';
    data[3] = 'D';
    data[4] = '\0';
    
    // Print original
    for (int i = 0; i < 4; i = i + 1) {
        putchar(data[i]);
    }
    putchar('\n');
    
    // Modify position 1 to 'X'
    modify_array(data, 1, 'X');
    
    // Print modified
    for (int i = 0; i < 4; i = i + 1) {
        putchar(data[i]);
    }
    putchar('\n');
    
    return 0;
}