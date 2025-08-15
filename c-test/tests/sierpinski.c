void putchar(int c);

int SIZE = 64;

int main() {
    int row = 0;
    while (row < SIZE) {
        int col = 0;

        // Print leading spaces
        int spaces = SIZE - row;
        while (spaces > 0) {
            putchar(' ');
            spaces = spaces - 1;
        }

        // Print stars using Pascal mod 2 trick
        while (col <= row) {
            if ((row & col) == col) {
                putchar('*');
            } else {
                putchar(' ');
            }
            putchar(' ');  // spacing between elements
            col = col + 1;
        }

        putchar('\n');
        row = row + 1;
    }

    return 0;
}