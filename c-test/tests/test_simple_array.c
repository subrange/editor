// Simple array test
void putchar(int c);

int main() {
    int arr[3];
    arr[0] = 10;
    arr[1] = 20;
    arr[2] = 30;
    
    // Direct access
    if (arr[0] == 10) putchar('1'); else putchar('N');
    if (arr[1] == 20) putchar('2'); else putchar('N');
    if (arr[2] == 30) putchar('3'); else putchar('N');
    
    putchar('\n');
    return 0;
}