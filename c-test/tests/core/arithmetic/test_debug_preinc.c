// Debug pre-increment more carefully
void putchar(int c);

int main() {
    int arr[5] = {10, 20, 30, 40, 50};
    int *p = arr;
    int *q;
    
    // Do pre-increment and save result
    q = ++p;  // q and p should both point to arr[1]
    
    // Check p
    putchar('P');
    putchar(':');
    putchar('0' + (*p / 10));
    putchar('0' + (*p % 10));
    putchar(' ');
    
    // Check q
    putchar('Q');
    putchar(':');
    putchar('0' + (*q / 10));
    putchar('0' + (*q % 10));
    putchar('\n');
    
    return 0;
}