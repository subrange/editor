// Test inline struct - simplified without putchar
int main() {
    struct {
        int x;
        int y;
    } p;
    
    p.x = 10;
    p.y = 20;
    
    // Direct output
    *(int*)0 = 'X';
    *(int*)0 = ':';
    *(int*)0 = '0' + p.x / 10;
    *(int*)0 = '0' + p.x % 10;
    *(int*)0 = ' ';
    
    *(int*)0 = 'Y';
    *(int*)0 = ':';
    *(int*)0 = '0' + p.y / 10;
    *(int*)0 = '0' + p.y % 10;
    *(int*)0 = '\n';
    
    return 0;
}