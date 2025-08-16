void putchar(int c);

#define DEFINED_MACRO

#ifdef DEFINED_MACRO
void print_yes() {
    putchar('Y');
}
#else
void print_yes() {
    putchar('N');
}
#endif

#ifndef UNDEFINED_MACRO
void print_yes2() {
    putchar('Y');
}
#else
void print_yes2() {
    putchar('N');
}
#endif

#ifdef UNDEFINED_MACRO
void print_no() {
    putchar('Y');
}
#else
void print_no() {
    putchar('N');
}
#endif

int main() {
    print_yes();
    print_yes2();
    print_no();
    putchar('\n');
    return 0;
}