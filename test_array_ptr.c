void putchar(int c);

int main() {
    int x = 5;
    int y = 6; 
    int z = 7;
    int *ptrs[3];
    
    ptrs[0] = &x;
    ptrs[1] = &y;
    ptrs[2] = &z;
    
    // Test accessing through array of pointers
    if (*ptrs[0] == 5) putchar('1'); else putchar('N');
    if (*ptrs[1] == 6) putchar('2'); else putchar('N');
    if (*ptrs[2] == 7) putchar('3'); else putchar('N');
    
    // Test modifying through array of pointers
    *ptrs[0] = 99;
    if (x == 99) putchar('4'); else putchar('N');
    
    putchar('\n');
    return 0;
}