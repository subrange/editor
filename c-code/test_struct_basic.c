// Test inline struct - most basic
int main() {
    struct {
        int x;
        int y;
    } p;
    
    p.x = 10;
    p.y = 20;
    
    int sum = p.x + p.y;
    
    return sum;
}