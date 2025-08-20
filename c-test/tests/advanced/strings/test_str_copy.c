#include <stdio.h>

void str_copy(char* dst, char* src) {
    int i = 0;
    while (src[i] && i < 31) {
        dst[i] = src[i];
        i++;
    }
    dst[i] = 0;
}

int main() {
    char * src = "ABC";
    char dst[32];
    str_copy(dst, src);

    puts(src);
    puts(dst);

    return 0;
}
