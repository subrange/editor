int puts(char* str);

void fizzbuzz(int max) {
    int i;
    for (i = 1; i <= max; i++) {
        if (i % 3 == 0 && i % 5 == 0) {
            puts("FizzBuzz");
        } else if (i % 3 == 0) {
            puts("Fizz");
        } else if (i % 5 == 0) {
            puts("Buzz");
        } else {
            int n = i;
            char buf[6];
            int pos = 0;
            char tmp[6];
            int tpos = 0;

            // Convert int to string manually
            if (n == 0) {
                buf[pos++] = '0';
            } else {
                if (n < 0) {
                    buf[pos++] = '-';
                    n = -n;
                }
                while (n > 0) {
                    tmp[tpos++] = '0' + (n % 10);
                    n /= 10;
                }
                while (tpos > 0) {
                    buf[pos++] = tmp[--tpos];
                }
            }
            buf[pos] = 0;
            puts(buf);
        }
    }
}

int main() {
    fizzbuzz(100);
    return 0;
}