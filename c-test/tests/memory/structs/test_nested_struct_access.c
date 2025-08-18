void putchar(int c);

struct Inner {
    int x;
    int y;
};

struct Outer {
    struct Inner inner;
};

int main() {
    struct Inner i;
    struct Outer o;

    i.x = 10;
    i.y = 20;

    o.inner = i; // This does not copy all the struct fields.

    if (o.inner.y == 20) {
        putchar('Y');
    } else {
        putchar('N');
    }

    putchar('\n');
    return 0;
}