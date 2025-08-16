void putchar(int c);

int main() {
    char list[10];
    int len = 3;

    list[0] = 'A';
    list[1] = 'B';
    list[2] = 'C';

    // Manually do what insert_char does for inserting 'X' at position 1
    // Step 1: Copy C from position 2 to position 3
    list[3] = list[2];
    
    // Step 2: Copy B from position 1 to position 2
    list[2] = list[1];
    
    // Step 3: Put 'X' at position 1
    list[1] = 'X';
    
    // Step 4: Update length
    len = 4;

    // Print result - should be AXBC
    putchar(list[0]);
    putchar(list[1]);
    putchar(list[2]);
    putchar(list[3]);
    putchar('\n');

    return 0;
}