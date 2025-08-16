void putchar(int c);

int main() {
    // Test 1: Simple AND
    int a = 1;
    int b = 1;

    // First case — simple check
    if (a == 1) {
        putchar('1');
    } else {
        putchar('N');
    }

    // Second case — simple check combining two conditions
    if (a == 1 && b == 1) {
        putchar('2');
    } else {
        putchar('N');
    }


    putchar('\n');
}