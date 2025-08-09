// Runtime implementation of memcpy
// Copies n bytes from src to dest

char *memcpy(char *dest, char *src, int n) {
    for (int i = 0; i < n; i = i + 1) {
        dest[i] = src[i];
    }
    
    return dest;
}