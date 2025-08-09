// Runtime implementation of memset
// Sets n bytes of memory to a given value

char *memset(char *s, int c, int n) {
    for (int i = 0; i < n; i = i + 1) {
        s[i] = c;  // c will be truncated to char automatically
    }
    
    return s;
}