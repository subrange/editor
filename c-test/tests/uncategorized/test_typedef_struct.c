typedef struct { 
    short integer; 
    unsigned short frac; 
} q16_16_t; 

q16_16_t test;

int main() {
    test.integer = 1;
    test.frac = 0;
    return 0;
}