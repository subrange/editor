// Simulated linked list without structs
// Two parallel arrays: values[] and next[]
// next[] stores the index of the next node, -1 for NULL
void putchar(int c);

int main() {
    int values[5];
    int next[5];

    // Build linked list manually:
    // Node 0 -> Node 1 -> Node 2 -> NULL
    values[0] = 'A';
    values[1] = 'B';
    values[2] = 'C';
    next[0] = 1;
    next[1] = 2;
    next[2] = -1;

    // The rest are unused
    values[3] = '?';
    values[4] = '?';
    next[3] = -1;
    next[4] = -1;

    // Traverse and print list
    int idx = 0;
    while (idx != -1) {
        putchar(values[idx]);
        idx = next[idx];
    }
    putchar('\n');

    // Insert new node 'X' between 'A' and 'B'
    values[3] = 'X';
    next[3] = next[0]; // old next of A
    next[0] = 3;

    // Traverse again
    idx = 0;
    while (idx != -1) {
        putchar(values[idx]);
        idx = next[idx];
    }
    putchar('\n');

    // Delete node after 'A' (which is now 'X')
    next[0] = next[next[0]]; // skip over node 3

    // Traverse final list
    idx = 0;
    while (idx != -1) {
        putchar(values[idx]);
        idx = next[idx];
    }
    putchar('\n');

    return 0;
}