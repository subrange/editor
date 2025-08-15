// printf for Ripple VM without varargs/const
// args[] is a pointer to ints
void putchar(int c);

void print_char(char c) {
    putchar((int)c);
}

void print_str(char *s) {
    while (*s) {
        putchar((int)*s++);
    }
}

void print_uint(unsigned int n, unsigned int base) {
    char buf[16];
    int i = 0;
    char *digits = "0123456789abcdef";

    if (n == 0) {
        print_char('0');
        return;
    }

    while (n > 0) {
        buf[i++] = digits[n % base];
        n /= base;
    }
    while (i--) {
        print_char(buf[i]);
    }
}

void print_int(int n) {
    if (n < 0) {
        print_char('-');
        print_uint((unsigned int)(-n), 10);
    } else {
        print_uint((unsigned int)n, 10);
    }
}

// fmt: format string
// args: pointer to an array of ints
void printf(char *fmt, int *args) {
    char *p = fmt;
    int argi = 0;

    while (*p) {
        if (*p == '%') {
            p++;
            if (*p == 's') {
                print_str((char *)args[argi++]);
            } else if (*p == 'c') {
                print_char((char)args[argi++]);
            } else if (*p == 'd') {
                print_int(args[argi++]);
            } else if (*p == 'x') {
                print_uint((unsigned int)args[argi++], 16);
            } else if (*p == '%') {
                print_char('%');
            } else {
                print_char('%');
                print_char(*p);
            }
        } else {
            print_char(*p);
        }
        p++;
    }
}